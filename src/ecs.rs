#[derive(Debug)]
enum DeadEntity {
    DeadEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entity {
    id: usize,
    generation: usize,
}

struct EntityGenerations {
    generations: Vec<usize>,
}

impl EntityGenerations {
    fn new() -> Self {
        Self {
            generations: Vec::new(),
        }
    }

    fn get(&self, entity_id: usize) -> usize {
        if entity_id >= self.generations.len() {
            return 0;
        }
        self.generations[entity_id]
    }

    fn increment(&mut self, entity_id: usize) -> usize {
        if entity_id >= self.generations.len() {
            // We make room for several extra entity ids to avoid
            // increasing the capacity by 1 over and over
            // and thus causing lots of copying.
            self.generations.resize(entity_id + 10, 0);
        }
        debug_assert!(entity_id < self.generations.len());
        self.generations[entity_id] += 1;
        self.generations[entity_id]
    }

    fn is_alive(&self, entity: Entity) -> bool {
        let entity_id = entity.id;
        let alive_generation: usize;
        if entity_id >= self.generations.len() {
            alive_generation = 0;
        } else {
            debug_assert!(entity_id < self.generations.len());
            alive_generation = self.generations[entity_id];
        }
        entity.generation == alive_generation
    }
}

struct ComponentPool<T: Clone> {
    components: Vec<(usize, Option<T>)>,
}

impl<T: Clone> ComponentPool<T> {
    fn new_one(entity: Entity, component: T) -> Self {
        // We make room for several extra components to avoid
        // increasing the capacity by 1 over and over
        // and thus causing lots of copying.
        let mut components = vec![(0, None); 10];
        components[entity.id] = (entity.generation, Some(component));
        Self { components }
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        if entity.id >= self.components.len() {
            return None;
        }
        let generation_component = &self.components[entity.id];
        if generation_component.0 < entity.generation {
            return None;
        }
        generation_component.1.as_ref()
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.id >= self.components.len() {
            return None;
        }
        let mut generation_component = &mut self.components[entity.id];
        if generation_component.0 < entity.generation {
            return None;
        }
        generation_component.1.as_mut()
    }

    fn set(&mut self, entity: Entity, component: T) {
        if entity.id >= self.components.len() {
            // We make room for several extra components to avoid
            // increasing the capacity by 1 over and over
            // and thus causing lots of copying.
            self.components.resize(entity.id + 10, (0, None));
        }
        self.components[entity.id] = (entity.generation, Some(component));
    }
}

trait System {
    fn required_components() -> Vec<std::any::TypeId>;
    fn run();
}

struct Registry {
    /// The maximum entity id we have issued. This is the "length" of the Registry.
    next_entity_id: usize,
    /// Entity ids that are free to issue again.
    free_entity_ids: Vec<usize>,
    /// The current generation of the entities.
    /// If a given Entity has a generation less than this,
    /// that Entity is no longer valid.
    entity_generations: EntityGenerations,
    /// The ComponentPools / Vectors that store components / values.
    component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
    systems: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            next_entity_id: 0,
            free_entity_ids: Vec::new(),
            entity_generations: EntityGenerations::new(),
            component_pools: std::collections::HashMap::new(),
            systems: std::collections::HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        if let Some(entity_id) = self.free_entity_ids.pop() {
            return Entity {
                id: entity_id,
                generation: self.entity_generations.get(entity_id),
            };
        }
        let new_entity = Entity {
            id: self.next_entity_id,
            generation: 0,
        };
        self.next_entity_id += 1;
        new_entity
    }

    fn is_entity_alive(&self, entity: Entity) -> bool {
        self.entity_generations.is_alive(entity)
    }

    fn remove_entity(&mut self, entity: Entity) -> Result<(), DeadEntity> {
        if !self.is_entity_alive(entity) {
            return Err(DeadEntity::DeadEntity);
        }
        self.entity_generations.increment(entity.id);
        self.free_entity_ids.push(entity.id);
        Ok(())
    }

    fn add_component<T: Clone + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), DeadEntity> {
        if !self.is_entity_alive(entity) {
            return Err(DeadEntity::DeadEntity);
        }
        let type_id = std::any::TypeId::of::<T>();
        match self.component_pools.get_mut(&type_id) {
            None => {
                let new_component_pool = Box::new(ComponentPool::<T>::new_one(entity, component));
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

    fn get_component<T: Clone + 'static>(&self, entity: Entity) -> Result<Option<&T>, DeadEntity> {
        if !self.is_entity_alive(entity) {
            return Err(DeadEntity::DeadEntity);
        }
        let type_id = std::any::TypeId::of::<T>();
        match self.component_pools.get(&type_id) {
            None => {
                return Ok(None);
            }
            Some(component_pool) => {
                let component_pool: &ComponentPool<T> = (&**component_pool).downcast_ref().unwrap();
                return Ok(component_pool.get(entity));
            }
        }
    }

    fn get_component_mut<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&mut T>, DeadEntity> {
        if !self.is_entity_alive(entity) {
            return Err(DeadEntity::DeadEntity);
        }
        let type_id = std::any::TypeId::of::<T>();
        match self.component_pools.get_mut(&type_id) {
            None => {
                return Ok(None);
            }
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                return Ok(component_pool.get_mut(entity));
            }
        }
    }

    fn add_system<T: System + 'static>(&mut self, system: T) {
        let type_id = std::any::TypeId::of::<T>();
        self.systems.insert(type_id, Box::new(system));
    }
}

#[test]
fn test_entity_generations_happy_path() {
    let mut eg = EntityGenerations::new();
    assert_eq!(eg.get(0), 0);
    assert_eq!(eg.get(1), 0);
    assert_eq!(eg.get(10), 0);
    assert_eq!(eg.get(5), 0);
    eg.increment(5);
    eg.increment(1);
    assert_eq!(eg.get(0), 0);
    assert_eq!(eg.get(1), 1);
    assert_eq!(eg.get(10), 0);
    assert_eq!(eg.get(5), 1);

    let mut eg = EntityGenerations::new();
    assert_eq!(eg.get(100), 0);
    eg.increment(100);
    assert_eq!(eg.get(100), 1);
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
    let e2: Entity = registry.create_entity();
    assert_eq!(registry.get_component::<i32>(e2).unwrap(), None);

    assert_eq!(registry.next_entity_id, 3);
    registry.remove_entity(e0).unwrap();
    registry.remove_entity(e1).unwrap();
    assert_eq!(registry.next_entity_id, 3);
    let _e0: Entity = registry.create_entity();
    let _e1: Entity = registry.create_entity();
    assert_eq!(registry.next_entity_id, 3);
    let _e3: Entity = registry.create_entity();
    assert_eq!(registry.next_entity_id, 4);

    registry.add_component(e2, 5_i32).unwrap();
    registry.remove_entity(e2).unwrap();
    assert!(registry.add_component(e2, 5_i32).is_err());
}

#[test]
fn test_system_happy_path() {
    #[derive(Clone)]
    struct CounterComponent {
        count: u32,
    }

    struct CounterIncrementSystem;

    impl System for CounterIncrementSystem {
        fn required_components() -> Vec<std::any::TypeId> {
            vec![std::any::TypeId::of::<CounterComponent>()]
        }

        fn run() {
            todo!()
        }
    }

    let mut registry = Registry::new();
    let e = registry.create_entity();
    registry
        .add_component(e, CounterComponent { count: 0 })
        .unwrap();
    registry.add_system(CounterIncrementSystem {});
}
