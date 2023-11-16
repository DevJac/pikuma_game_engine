#[derive(Debug)]
enum DeadEntity {
    DeadEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entity {
    id: usize,
    generation: usize,
}

struct FreeEntityIds {
    ids: Vec<usize>,
}

impl FreeEntityIds {
    fn new() -> Self {
        Self { ids: Vec::new() }
    }

    fn push(&mut self, id: usize) {
        self.ids.push(id)
    }

    fn pop(&mut self) -> Option<usize> {
        self.ids.pop()
    }
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
    components: Vec<Option<T>>,
}

impl<T: Clone> ComponentPool<T> {
    fn new_one(entity_id: usize, component: T) -> Self {
        // We make room for several extra components to avoid
        // increasing the capacity by 1 over and over
        // and thus causing lots of copying.
        let mut components = vec![None; 10];
        components[entity_id] = Some(component);
        Self { components }
    }

    fn get(&self, entity_id: usize) -> Option<&T> {
        if entity_id >= self.components.len() {
            return None;
        }
        self.components[entity_id].as_ref()
    }

    fn set(&mut self, entity_id: usize, component: T) {
        if entity_id >= self.components.len() {
            // We make room for several extra components to avoid
            // increasing the capacity by 1 over and over
            // and thus causing lots of copying.
            self.components.resize(entity_id + 10, None);
        }
        self.components[entity_id] = Some(component);
    }
}

struct Registry {
    /// The maximum entity id we have issued. This is the "length" of the Registry.
    next_entity_id: usize,
    /// Entity ids that are free to issue again.
    free_entity_ids: FreeEntityIds,
    /// The current generation of the entities.
    /// If a given Entity has a generation less than this,
    /// that Entity is no longer valid.
    entity_generations: EntityGenerations,
    /// The ComponentPools / Vectors that store components / values.
    component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            next_entity_id: 0,
            free_entity_ids: FreeEntityIds::new(),
            entity_generations: EntityGenerations::new(),
            component_pools: std::collections::HashMap::new(),
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
        let type_id = (&component as &dyn std::any::Any).type_id();
        match self.component_pools.get_mut(&type_id) {
            None => {
                let new_component_pool =
                    Box::new(ComponentPool::<T>::new_one(entity.id, component));
                self.component_pools.insert(type_id, new_component_pool);
            }
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                component_pool.set(entity.id, component);
            }
        }
        Ok(())
    }

    fn get_component<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&T>, DeadEntity> {
        if !self.is_entity_alive(entity) {
            return Err(DeadEntity::DeadEntity);
        }
        let type_id: std::any::TypeId = std::any::TypeId::of::<T>();
        match self.component_pools.get_mut(&type_id) {
            None => {
                return Ok(None);
            }
            Some(component_pool) => {
                let component_pool: &ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                return Ok(component_pool.get(entity.id));
            }
        }
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
