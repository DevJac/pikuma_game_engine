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
    entities: std::collections::HashSet<crate::ecs::Entity>,
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

impl crate::ecs::SystemBase for MovementSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.remove(&entity);
    }
}

impl crate::ecs::System<f32> for MovementSystem {
    fn run(&self, ec_manager: &mut crate::ecs::EntityComponentWrapper, delta_time: f32) {
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
    pub tank_or_tree: crate::renderer::TankOrTree,
}

pub struct RenderSystem {
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<crate::ecs::Entity>,
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

impl crate::ecs::SystemBase for RenderSystem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.remove(&entity);
    }
}

impl crate::ecs::System<&mut crate::renderer::Renderer> for RenderSystem {
    fn run(
        &self,
        ec_manager: &mut crate::ecs::EntityComponentWrapper,
        renderer: &mut crate::renderer::Renderer,
    ) {
        for entity in self.entities.iter() {
            let rigid_body_component: &RigidBodyComponent =
                ec_manager.get_component(*entity).unwrap().unwrap();
            let sprite_component: &SpriteComponent =
                ec_manager.get_component(*entity).unwrap().unwrap();
            renderer.draw_image(
                sprite_component.tank_or_tree,
                rigid_body_component.position.as_uvec2(),
            );
        }
    }
}
