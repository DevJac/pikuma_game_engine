use crate::ecs::{Entity, EntityComponentWrapper, System, SystemBase};

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

#[derive(Clone)]
pub struct SpriteComponent {
    pub sprite_index: crate::renderer::SpriteIndex,
    pub sprite_z: f32,
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
    type Input<'i> = &'i mut crate::renderer::Renderer;

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
        components.sort_by(|a, b| a.1.sprite_z.partial_cmp(&b.1.sprite_z).unwrap());
        for (rigid_body_component, sprite_component) in components {
            renderer.draw_image(
                sprite_component.sprite_index,
                sprite_component.sprite_z,
                rigid_body_component.position.as_uvec2(),
            );
        }
    }
}
