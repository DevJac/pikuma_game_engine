///////////////////////////////////////////////////////////////////////////////
// RigidBody / Movement
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct RigidBody {
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
        required_components.insert(std::any::TypeId::of::<RigidBody>());
        Self {
            required_components,
            entities: std::collections::HashSet::new(),
        }
    }
}

impl crate::ecs::System for MovementSystem {
    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
        &self.required_components
    }

    fn add_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.insert(entity);
    }

    fn remove_entity(&mut self, entity: crate::ecs::Entity) {
        self.entities.remove(&entity);
    }

    fn run(&self, ec_manager: &mut crate::ecs::EntityComponentWrapper) {
        for entity in self.entities.iter() {
            let rigid_body_component: &mut RigidBody =
                ec_manager.get_component_mut(*entity).unwrap().unwrap();
            rigid_body_component.position += rigid_body_component.velocity;
        }
    }
}
