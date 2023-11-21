type IndexT = u32;
type GenerationT = u32;

const VEC_RESIZE_MARGIN: usize = 10;

#[derive(Debug)]
enum EcsError {
    DeadEntity,
    NoSuchComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
struct Entity {
    id: IndexT,
    generation: GenerationT,
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.generation.cmp(&other.generation))
    }
}

struct EntityManager {
    /// Entity ids that are free to issue again.
    free_entity_ids: Vec<IndexT>,
    /// When we need a new entity id, because there are no free entity ids, use this one.
    next_entity_id: IndexT,
    /// The current generation of the entities.
    /// If a given Entity has a generation less than this,
    /// that Entity is no longer valid.
    generations: Vec<IndexT>,
}

impl EntityManager {
    fn new() -> Self {
        Self {
            free_entity_ids: Vec::new(),
            next_entity_id: 0,
            generations: Vec::new(),
        }
    }

    /// Returns a free entity id if available, otherwise returns the next new entity id.
    /// Will not alter generations, which should only be altered when removing entities.
    fn create_entity(&mut self) -> Entity {
        if let Some(entity_id) = self.free_entity_ids.pop() {
            return Entity {
                id: entity_id,
                generation: self.alive_generation(entity_id),
            };
        }
        let result = Entity {
            id: self.next_entity_id,
            generation: self.alive_generation(self.next_entity_id),
        };
        self.next_entity_id += 1;
        result
    }

    /// Removes entity by incrementing the generation.
    /// Stores free entity id to be reused.
    /// Returns an Err Result if entity already removed / dead.
    fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
        if self.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let entity_id = entity.id as usize;
        if entity_id >= self.generations.len() {
            self.generations.resize(entity_id + VEC_RESIZE_MARGIN, 0);
        }
        self.free_entity_ids.push(entity.id);
        self.generations[entity_id] += 1;
        Ok(())
    }

    fn is_alive(&self, entity: Entity) -> bool {
        entity.generation == self.alive_generation(entity.id)
    }

    fn is_dead(&self, entity: Entity) -> bool {
        entity.generation != self.alive_generation(entity.id)
    }

    fn alive_generation(&self, entity_id: IndexT) -> GenerationT {
        let entity_id = entity_id as usize;
        if entity_id >= self.generations.len() {
            return 0;
        }
        self.generations[entity_id]
    }
}

struct ComponentPool<T: Clone> {
    components: Vec<(IndexT, Option<T>)>,
}

impl<T: Clone> ComponentPool<T> {
    fn new_one(entity: Entity, component: T) -> Self {
        // We make room for several extra components to avoid
        // increasing the capacity by 1 over and over
        // and thus causing lots of copying.
        let mut components = vec![(0, None); VEC_RESIZE_MARGIN];
        components[entity.id as usize] = (entity.generation, Some(component));
        Self { components }
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        if entity.id as usize >= self.components.len() {
            return None;
        }
        let generation_component = &self.components[entity.id as usize];
        if generation_component.0 < entity.generation {
            return None;
        }
        generation_component.1.as_ref()
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.id as usize >= self.components.len() {
            return None;
        }
        let generation_component = &mut self.components[entity.id as usize];
        if generation_component.0 < entity.generation {
            return None;
        }
        generation_component.1.as_mut()
    }

    fn set(&mut self, entity: Entity, component: T) {
        if entity.id as usize >= self.components.len() {
            // We make room for several extra components to avoid
            // increasing the capacity by 1 over and over
            // and thus causing lots of copying.
            self.components
                .resize(entity.id as usize + VEC_RESIZE_MARGIN, (0, None));
        }
        self.components[entity.id as usize] = (entity.generation, Some(component));
    }

    fn remove(&mut self, entity: Entity) {
        if entity.id as usize >= self.components.len() {
            return;
        }
        self.components[entity.id as usize] = (entity.generation, None);
    }
}

struct EntityComponentManager {
    entity_manager: EntityManager,
    entity_components:
        std::collections::HashMap<Entity, std::collections::HashSet<std::any::TypeId>>,
    component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl EntityComponentManager {
    fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            entity_components: std::collections::HashMap::new(),
            component_pools: std::collections::HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        let new_entity = self.entity_manager.create_entity();
        self.entity_components
            .insert(new_entity, std::collections::HashSet::new());
        new_entity
    }

    fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
        self.entity_components.remove(&entity);
        self.entity_manager.remove_entity(entity)
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.entity_manager.is_alive(entity)
    }

    fn is_dead(&self, entity: Entity) -> bool {
        self.entity_manager.is_dead(entity)
    }

    fn add_component<T: Clone + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        if self.entity_manager.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        self.entity_components
            .get_mut(&entity)
            .unwrap()
            .insert(type_id);
        match self.component_pools.get_mut(&type_id) {
            None => {
                let new_component_pool = Box::new(ComponentPool::new_one(entity, component));
                self.component_pools.insert(type_id, new_component_pool);
            }
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                component_pool.set(entity, component);
            }
        }
        Ok(())
    }

    fn remove_component<T: Clone + 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        if self.entity_manager.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        self.entity_components
            .get_mut(&entity)
            .unwrap()
            .remove(&type_id);
        match self.component_pools.get_mut(&type_id) {
            None => {
                return Err(EcsError::NoSuchComponent);
            }
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                component_pool.remove(entity);
            }
        }
        Ok(())
    }

    fn get_component<T: Clone + 'static>(&self, entity: Entity) -> Result<Option<&T>, EcsError> {
        if self.entity_manager.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        match self.component_pools.get(&type_id) {
            None => Err(EcsError::NoSuchComponent),
            Some(component_pool) => {
                let component_pool: &ComponentPool<T> = (&**component_pool).downcast_ref().unwrap();
                Ok(component_pool.get(entity))
            }
        }
    }

    fn get_component_mut<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&mut T>, EcsError> {
        if self.entity_manager.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        match self.component_pools.get_mut(&type_id) {
            None => Err(EcsError::NoSuchComponent),
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                Ok(component_pool.get_mut(entity))
            }
        }
    }

    fn has_components(&self, entity: Entity) -> &std::collections::HashSet<std::any::TypeId> {
        self.entity_components.get(&entity).unwrap()
    }

    fn entities_and_components(
        &self,
    ) -> impl Iterator<Item = (&Entity, &std::collections::HashSet<std::any::TypeId>)> {
        self.entity_components.iter()
    }
}

trait System {
    fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId>;
    fn add_entity(&mut self, entity: Entity);
    fn remove_entity(&mut self, entity: Entity);
    fn run(&self, ec_manager: &mut EntityComponentManager);
}

struct Registry {
    ec_manager: EntityComponentManager,
    systems: std::collections::HashMap<std::any::TypeId, Box<dyn System>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            ec_manager: EntityComponentManager::new(),
            systems: std::collections::HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        // Because a new entity has no components, no systems will be interested in it.
        self.ec_manager.create_entity()
    }

    fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
        for system in self.systems.values_mut() {
            system.remove_entity(entity);
        }
        self.ec_manager.remove_entity(entity)
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.ec_manager.is_alive(entity)
    }

    fn is_dead(&self, entity: Entity) -> bool {
        self.ec_manager.is_dead(entity)
    }

    fn add_component<T: Clone + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        let result = self.ec_manager.add_component(entity, component);
        if result.is_ok() {
            for system in self.systems.values_mut() {
                if self
                    .ec_manager
                    .has_components(entity)
                    .is_superset(system.required_components())
                {
                    system.add_entity(entity);
                }
            }
        }
        result
    }

    fn remove_component<T: Clone + 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        let result = self.ec_manager.remove_component::<T>(entity);
        if result.is_ok() {
            for system in self.systems.values_mut() {
                if self
                    .ec_manager
                    .has_components(entity)
                    .is_superset(system.required_components())
                {
                    system.remove_entity(entity);
                }
            }
        }
        result
    }

    fn get_component<T: Clone + 'static>(&self, entity: Entity) -> Result<Option<&T>, EcsError> {
        self.ec_manager.get_component(entity)
    }

    fn get_component_mut<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&mut T>, EcsError> {
        self.ec_manager.get_component_mut(entity)
    }

    fn add_system<T: System + 'static>(&mut self, mut system: T) {
        for (entity, components) in self.ec_manager.entities_and_components() {
            if components.is_superset(system.required_components()) {
                system.add_entity(*entity);
            }
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        self.systems.insert(type_id, Box::new(system));
    }

    fn remove_system<T: System + 'static>(&mut self) {
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        self.systems.remove(&type_id);
    }

    fn run_systems(&mut self) {
        for system in self.systems.values() {
            system.run(&mut self.ec_manager);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityComponentManager, EntityManager, Registry, System};

    #[test]
    fn test_entity_manager_happy_path() {
        let mut em = EntityManager::new();

        let e0: Entity = em.create_entity();
        let e1: Entity = em.create_entity();
        let e2: Entity = em.create_entity();

        assert_eq!(e0.id, 0);
        assert_eq!(e1.id, 1);
        assert_eq!(e2.id, 2);

        assert!(em.is_alive(e0));
        assert!(em.is_alive(e1));
        assert!(em.is_alive(e2));
        em.remove_entity(e1).unwrap();
        assert!(em.is_alive(e0));
        assert!(em.is_dead(e1));
        assert!(em.is_alive(e2));

        assert!(em.remove_entity(e1).is_err());

        let e3: Entity = em.create_entity();
        assert_eq!(e3.id, 1);
        assert!(em.is_alive(e0));
        assert!(em.is_dead(e1));
        assert!(em.is_alive(e2));
        assert!(em.is_alive(e3));

        assert!(em.remove_entity(e1).is_err());
    }

    #[test]
    fn test_registry_happy_path() {
        let mut registry: Registry = Registry::new();
        let e0: Entity = registry.create_entity();
        let e1: Entity = registry.create_entity();

        let e2: Entity = registry.create_entity();
        registry.add_component(e2, 5_i32).unwrap();
        assert_eq!(registry.get_component::<i32>(e2).unwrap().unwrap(), &5_i32);
        registry.remove_entity(e2).unwrap();
        assert!(registry.get_component::<i32>(e2).is_err());
        let e2: Entity = registry.create_entity();
        assert_eq!(registry.get_component::<i32>(e2).unwrap(), None);

        assert_eq!(registry.ec_manager.entity_manager.next_entity_id, 3);
        registry.remove_entity(e0).unwrap();
        registry.remove_entity(e1).unwrap();
        assert_eq!(registry.ec_manager.entity_manager.next_entity_id, 3);
        let _e0: Entity = registry.create_entity();
        let _e1: Entity = registry.create_entity();
        assert_eq!(registry.ec_manager.entity_manager.next_entity_id, 3);
        let _e3: Entity = registry.create_entity();
        assert_eq!(registry.ec_manager.entity_manager.next_entity_id, 4);

        registry.add_component(e2, 5_i32).unwrap();
        registry.remove_entity(e2).unwrap();
        assert!(registry.add_component(e2, 5_i32).is_err());
    }

    #[derive(Clone)]
    struct CounterComponent {
        count: u32,
    }

    struct CounterIncrementSystem {
        required_components: std::collections::HashSet<std::any::TypeId>,
        entities: std::collections::HashSet<Entity>,
    }

    impl CounterIncrementSystem {
        fn new() -> Self {
            let mut required_components = std::collections::HashSet::new();
            required_components.insert(std::any::TypeId::of::<CounterComponent>());
            Self {
                required_components,
                entities: std::collections::HashSet::new(),
            }
        }
    }

    impl System for CounterIncrementSystem {
        fn required_components(&self) -> &std::collections::HashSet<std::any::TypeId> {
            &self.required_components
        }

        fn add_entity(&mut self, entity: Entity) {
            self.entities.insert(entity);
        }

        fn remove_entity(&mut self, entity: Entity) {
            self.entities.remove(&entity);
        }

        fn run(&self, ec_manager: &mut EntityComponentManager) {
            for entity in self.entities.iter() {
                let counter_component: &mut CounterComponent =
                    ec_manager.get_component_mut(*entity).unwrap().unwrap();
                counter_component.count += 1;
            }
        }
    }

    #[test]
    fn test_system_happy_path() {
        let mut registry = Registry::new();
        let e = registry.create_entity();
        registry
            .add_component(e, CounterComponent { count: 0 })
            .unwrap();
        registry.add_system(CounterIncrementSystem::new());
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            0
        );
        registry.run_systems();
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            1
        );
        registry.run_systems();
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            2
        );
    }
}