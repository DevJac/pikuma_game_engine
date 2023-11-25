///////////////////////////////////////////////////////////////////////////////
// RigidBody / Movement
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct RigidBodyComponent {
    position: glam::Vec2,
    velocity: glam::Vec2,
}

struct MovementSystem {
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<crate::ecs::Entity>,
}

impl MovementSystem {
    fn new() -> Self {
        let mut required_components = std::collections::HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl crate::ecs::System for MovementSystem {
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

    fn run(
        &self,
        ec_manager: &mut crate::ecs::EntityComponentWrapper,
        delta_time: &dyn std::any::Any,
    ) {
        let delta_time: &f32 = delta_time
            .downcast_ref()
            .expect("MovementSystem expects u32 delta_time");
        for entity in self.entities.iter() {
            let rigid_body_component: &mut RigidBodyComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            rigid_body_component.position += rigid_body_component.velocity * *delta_time;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Sprite / Render
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct SpriteComponent {
    tank_or_tree: crate::renderer::TankOrTree,
}

struct RenderSystem {
    required_components: std::collections::HashSet<std::any::TypeId>,
    entities: std::collections::HashSet<crate::ecs::Entity>,
}

impl RenderSystem {
    fn new() -> Self {
        let mut required_components = std::collections::HashSet::new();
        required_components.insert(std::any::TypeId::of::<RigidBodyComponent>());
        required_components.insert(std::any::TypeId::of::<SpriteComponent>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl crate::ecs::System for RenderSystem {
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

    fn run(
        &self,
        ec_manager: &mut crate::ecs::EntityComponentWrapper,
        renderer: &dyn std::any::Any,
    ) {
        let renderer: &crate::renderer::Renderer = renderer
            .downcast_ref()
            .expect("RenderSystem expects renderer");
        for entity in self.entities.iter() {
            let rigid_body_component: &mut RigidBodyComponent =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            let sprite_component: &SpriteComponent =
                ec_manager.get_component(*entity).unwrap().unwrap();
        }
    }
}
