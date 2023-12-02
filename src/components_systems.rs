use crate::{
    ecs::{Entity, EntityComponentWrapper, System, SystemBase},
    renderer::{Renderer, SpriteIndex},
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
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<Entity>,
}

impl MovementSystem {
    pub fn new() -> Self {
        let mut required_components = std::collections::HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl SystemBase for MovementSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
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
}

pub struct RenderSystem {
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<Entity>,
}

impl RenderSystem {
    pub fn new() -> Self {
        let mut required_components = std::collections::HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl SystemBase for RenderSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
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
                rigid_body_component.position.as_uvec2(),
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
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<Entity>,
}

impl AnimationSystem {
    pub fn new() -> Self {
        let mut required_components = std::collections::HashSet::new();
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        required_components.insert(std::any::TypeId::of::<AnimationComponent>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl SystemBase for AnimationSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
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
