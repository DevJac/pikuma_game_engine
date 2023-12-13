use std::collections::HashSet;

use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{
    ecs::{Entity, EntityComponentWrapper, System, SystemBase},
    event_bus::{Handler, HandlerBase},
    renderer::{Camera, Renderer, SpriteIndex},
};

///////////////////////////////////////////////////////////////////////////////
// RigidBody / Movement
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RigidBodyComponent {
    pub position: glam::Vec2,
    pub velocity: glam::Vec2,
}

pub struct MovementSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
}

impl MovementSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
        }
    }
}

impl SystemBase for MovementSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for MovementSystem {
    type Input<'i> = f32;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, delta_time: Self::Input<'_>) {
        for entity in self.entities.iter() {
            let rigid_body_component: &mut RigidBodyComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            rigid_body_component.position += rigid_body_component.velocity * delta_time;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Sprite / Render
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Ground,
    Air,
}

impl Layer {
    fn as_z(&self) -> f32 {
        match self {
            Layer::Background => 0.0,
            Layer::Ground => 0.5,
            Layer::Air => 1.0,
        }
    }
}

#[derive(Clone)]
pub struct SpriteComponent {
    pub sprite_index: SpriteIndex,
    pub sprite_layer: Layer,
    pub size: glam::Vec2,
}

pub struct RenderSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
}

impl RenderSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
        }
    }
}

impl SystemBase for RenderSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for RenderSystem {
    type Input<'i> = &'i mut Renderer;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, renderer: Self::Input<'_>) {
        let mut components: Vec<(&RigidBodyComponent, &SpriteComponent)> = self
            .entities
            .iter()
            .map(|entity| {
                let rigid_body_component: &RigidBodyComponent =
                    ec_manager.get_component(*entity).unwrap().unwrap();
                let sprite_component: &SpriteComponent =
                    ec_manager.get_component(*entity).unwrap().unwrap();
                (rigid_body_component, sprite_component)
            })
            .collect();
        components.sort_by(|a, b| {
            a.1.sprite_layer
                .as_z()
                .partial_cmp(&b.1.sprite_layer.as_z())
                .unwrap()
        });
        for (rigid_body_component, sprite_component) in components {
            renderer.draw_image(
                sprite_component.sprite_index,
                sprite_component.sprite_layer.as_z(),
                rigid_body_component.position,
                sprite_component.size,
            );
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Animation
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct AnimationComponent {
    pub frames: Vec<SpriteIndex>,
    pub frame_time: f32,
    pub current_frame: u32,
    pub current_frame_time: f32,
}

impl AnimationComponent {
    pub fn new(frame_time: f32, frames: Vec<SpriteIndex>) -> Self {
        Self {
            frames,
            frame_time,
            current_frame: 0,
            current_frame_time: 0.0,
        }
    }
}

pub struct AnimationSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
}

impl AnimationSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        required_components.insert(std::any::TypeId::of::<AnimationComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
        }
    }
}

impl SystemBase for AnimationSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for AnimationSystem {
    type Input<'i> = f32;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, delta_time: Self::Input<'_>) {
        for entity in self.entities.iter() {
            let animation_component: &mut AnimationComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            animation_component.current_frame_time += delta_time;
            let mut update_sprite_frame: Option<SpriteIndex> = None;
            if animation_component.current_frame_time > animation_component.frame_time {
                animation_component.current_frame_time -= animation_component.frame_time;
                animation_component.current_frame = (animation_component.current_frame + 1)
                    % animation_component.frames.len() as u32;
                update_sprite_frame =
                    Some(animation_component.frames[animation_component.current_frame as usize]);
            }
            if let Some(update_sprite_frame) = update_sprite_frame {
                let sprite_component: &mut SpriteComponent =
                    ec_manager.get_component_mut(*entity).unwrap().unwrap();
                sprite_component.sprite_index = update_sprite_frame;
            }
        }
    }
}

#[derive(Clone)]
pub struct MotionAnimationComponent {
    pub left_frames: Vec<SpriteIndex>,
    pub down_frames: Vec<SpriteIndex>,
    pub right_frames: Vec<SpriteIndex>,
    pub up_frames: Vec<SpriteIndex>,
    pub last_velocity: glam::Vec2,
    pub frame_time: f32,
    pub current_frame: u32,
    pub current_frame_time: f32,
}

impl MotionAnimationComponent {
    pub fn new(
        frame_time: f32,
        left_frames: Vec<SpriteIndex>,
        down_frames: Vec<SpriteIndex>,
        right_frames: Vec<SpriteIndex>,
        up_frames: Vec<SpriteIndex>,
    ) -> Self {
        Self {
            left_frames,
            down_frames,
            right_frames,
            up_frames,
            frame_time,
            current_frame: 0,
            current_frame_time: 0.0,
            last_velocity: glam::Vec2::ZERO,
        }
    }
}

pub struct MotionAnimationSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
}

impl MotionAnimationSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        required_components.insert(std::any::TypeId::of::<MotionAnimationComponent>());
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
        }
    }
}

impl SystemBase for MotionAnimationSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for MotionAnimationSystem {
    type Input<'i> = f32;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, delta_time: Self::Input<'_>) {
        for entity in self.entities.iter() {
            let rigid_body_component: &RigidBodyComponent =
                ec_manager.get_component(*entity).unwrap().unwrap();
            let mut velocity = rigid_body_component.velocity;
            let motion_animation_component: &mut MotionAnimationComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            if velocity == glam::Vec2::ZERO {
                velocity = motion_animation_component.last_velocity;
            }
            motion_animation_component.last_velocity = velocity;
            let cardinal_frames = [
                (
                    glam::Vec2::new(0.0, 1.0),
                    &motion_animation_component.down_frames,
                ),
                (
                    glam::Vec2::new(1.0, 0.0),
                    &motion_animation_component.right_frames,
                ),
                (
                    glam::Vec2::new(-1.0, 0.0),
                    &motion_animation_component.left_frames,
                ),
                (
                    glam::Vec2::new(0.0, -1.0),
                    &motion_animation_component.up_frames,
                ),
            ];
            let (_, frames) = cardinal_frames
                .iter()
                .max_by(|(dir0, _), (dir1, _)| {
                    let dot0 = velocity.dot(*dir0);
                    let dot1 = velocity.dot(*dir1);
                    dot0.partial_cmp(&dot1).unwrap()
                })
                .unwrap();
            motion_animation_component.current_frame_time += delta_time;
            let mut update_sprite_frame: Option<SpriteIndex> = None;
            if motion_animation_component.current_frame_time > motion_animation_component.frame_time
            {
                motion_animation_component.current_frame_time -=
                    motion_animation_component.frame_time;
                motion_animation_component.current_frame =
                    (motion_animation_component.current_frame + 1) % frames.len() as u32;
                update_sprite_frame =
                    Some(frames[motion_animation_component.current_frame as usize]);
            }
            if let Some(update_sprite_frame) = update_sprite_frame {
                let sprite_component: &mut SpriteComponent =
                    ec_manager.get_component_mut(*entity).unwrap().unwrap();
                sprite_component.sprite_index = update_sprite_frame;
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Collision
///////////////////////////////////////////////////////////////////////////////

pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

pub struct Rectangle {
    top_left: glam::Vec2,
    bottom_right: glam::Vec2,
}

impl Rectangle {
    fn range_intersects(a0: f32, a1: f32, b0: f32, b1: f32) -> bool {
        (a0 <= b0 && b0 <= a1) || (a0 <= b1 && b1 <= a1) || (b0 <= a0 && a0 <= b1)
    }

    fn collides_with(&self, other: &Rectangle) -> bool {
        let x_axis_intersects = Self::range_intersects(
            self.top_left.x,
            self.bottom_right.x,
            other.top_left.x,
            other.bottom_right.x,
        );
        let y_axis_intersects = Self::range_intersects(
            self.top_left.y,
            self.bottom_right.y,
            other.top_left.y,
            other.bottom_right.y,
        );
        x_axis_intersects && y_axis_intersects
    }
}

#[derive(Clone)]
pub struct CollisionComponent {
    pub offset: glam::Vec2,
    pub width_height: glam::Vec2,
}

pub struct CollisionSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
    render_collision_boxes: bool,
}

impl CollisionSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<CollisionComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
            render_collision_boxes: false,
        }
    }
}

impl SystemBase for CollisionSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for CollisionSystem {
    type Input<'i> = &'i mut Renderer;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, renderer: Self::Input<'_>) {
        let entities: Vec<&Entity> = self.entities.iter().collect();
        for a_index in 0..entities.len() {
            let entity_a = entities[a_index];
            if ec_manager.is_dead(*entity_a) {
                continue;
            }
            let rigid_body_a: &RigidBodyComponent =
                ec_manager.get_component(*entity_a).unwrap().unwrap();
            let collision_a: &CollisionComponent =
                ec_manager.get_component(*entity_a).unwrap().unwrap();
            if self.render_collision_boxes {
                renderer.draw_rectangle(
                    rigid_body_a.position + collision_a.offset,
                    collision_a.width_height,
                );
            }
            let world_space_collision_rectangle_a = Rectangle {
                top_left: rigid_body_a.position + collision_a.offset,
                bottom_right: rigid_body_a.position + collision_a.offset + collision_a.width_height,
            };
            for b_index in (a_index + 1)..entities.len() {
                let entity_b = entities[b_index];
                if ec_manager.is_dead(*entity_b) {
                    continue;
                }
                let rigid_body_b: &RigidBodyComponent =
                    ec_manager.get_component(*entity_b).unwrap().unwrap();
                let collision_b: &CollisionComponent =
                    ec_manager.get_component(*entity_b).unwrap().unwrap();
                let world_space_collision_rectangle_b = Rectangle {
                    top_left: rigid_body_b.position + collision_b.offset,
                    bottom_right: rigid_body_b.position
                        + collision_b.offset
                        + collision_b.width_height,
                };
                if world_space_collision_rectangle_a
                    .collides_with(&world_space_collision_rectangle_b)
                {
                    ec_manager.dispatch_event(CollisionEvent {
                        entity_a: *entity_a,
                        entity_b: *entity_b,
                    });
                }
            }
        }
    }
}

impl HandlerBase for CollisionSystem {
    fn handle_any(&mut self, ec_manager: &mut EntityComponentWrapper, event: &dyn std::any::Any) {
        if let Some(event) = event.downcast_ref::<CollisionEvent>() {
            self.handle(ec_manager, event);
        }
        if let Some(event) = event.downcast_ref::<PhysicalKey>() {
            self.handle(ec_manager, event);
        }
    }
}

impl Handler<CollisionEvent> for CollisionSystem {
    fn handle(
        &mut self,
        ec_manager: &mut EntityComponentWrapper,
        collision_event: &CollisionEvent,
    ) {
        ec_manager.remove_entity(collision_event.entity_a).unwrap();
        ec_manager.remove_entity(collision_event.entity_b).unwrap();
    }
}

impl Handler<PhysicalKey> for CollisionSystem {
    fn handle(&mut self, _ec_manager: &mut EntityComponentWrapper, event: &PhysicalKey) {
        if let PhysicalKey::Code(KeyCode::KeyB) = event {
            self.render_collision_boxes = !self.render_collision_boxes;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Keyboard Control
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct KeyboardControlComponent;

pub struct KeyboardControlSystem {
    required_components: HashSet<std::any::TypeId>,
    entities: HashSet<Entity>,
}

impl KeyboardControlSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<KeyboardControlComponent>());
        Self {
            required_components,
            entities: HashSet::new(),
        }
    }
}

impl SystemBase for KeyboardControlSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }
}

impl System for KeyboardControlSystem {
    type Input<'i> = &'i HashSet<PhysicalKey>;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, pressed_keys: Self::Input<'_>) {
        let mut unit_velocity = glam::Vec2::ZERO;
        if pressed_keys.contains(&PhysicalKey::Code(KeyCode::KeyA)) {
            unit_velocity += glam::Vec2::new(-1.0, 0.0);
        }
        if pressed_keys.contains(&PhysicalKey::Code(KeyCode::KeyS)) {
            unit_velocity += glam::Vec2::new(0.0, 1.0);
        }
        if pressed_keys.contains(&PhysicalKey::Code(KeyCode::KeyD)) {
            unit_velocity += glam::Vec2::new(1.0, 0.0);
        }
        if pressed_keys.contains(&PhysicalKey::Code(KeyCode::KeyW)) {
            unit_velocity += glam::Vec2::new(0.0, -1.0);
        }
        let velocity = unit_velocity * 80.0;
        for entity in self.entities.iter() {
            let rigid_body_component: &mut RigidBodyComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            rigid_body_component.velocity = velocity;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Camera
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CameraFocusComponent {
    pub focus_offset: glam::Vec2,
    pub viewport_size: glam::Vec2,
    pub map_top_left: glam::Vec2,
    pub map_bottom_right: glam::Vec2,
}

pub struct CameraFocusSystem {
    required_components: HashSet<std::any::TypeId>,
    entity: Option<Entity>,
}

impl CameraFocusSystem {
    pub fn new() -> Self {
        let mut required_components = HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<CameraFocusComponent>());
        Self {
            required_components,
            entity: None,
        }
    }
}

impl SystemBase for CameraFocusSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    fn remove_entity(&mut self, entity: Entity) {
        if self.entity == Some(entity) {
            self.entity = None;
        }
    }
}

impl System for CameraFocusSystem {
    type Input<'i> = &'i mut Renderer;

    fn run(&self, ec_manager: &mut EntityComponentWrapper, renderer: Self::Input<'_>) {
        if self.entity.is_none() {
            return;
        }
        let entity = self.entity.unwrap();
        let rigid_body_component: &RigidBodyComponent =
            ec_manager.get_component(entity).unwrap().unwrap();
        let camera_focus_component: &CameraFocusComponent =
            ec_manager.get_component(entity).unwrap().unwrap();
        // TODO: Constrain viewport at edges of map
        let focus = rigid_body_component.position + camera_focus_component.focus_offset;
        let focus_top_left = focus - (camera_focus_component.viewport_size / 2.0);
        let focus_top_left_out_of_bounds =
            (camera_focus_component.map_top_left - focus_top_left).max(glam::Vec2::ZERO);
        let focus_bottom_right = focus + (camera_focus_component.viewport_size / 2.0);
        let focus_bottom_right_out_of_bounds =
            (camera_focus_component.map_bottom_right - focus_bottom_right).min(glam::Vec2::ZERO);
        let camera = Camera {
            top_left: focus_top_left
                + focus_top_left_out_of_bounds
                + focus_bottom_right_out_of_bounds,
            width_height: camera_focus_component.viewport_size,
        };
        renderer.set_camera(camera);
    }
}
