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

    fn get(&mut self, entity_id: usize) -> usize {
        if entity_id >= self.generations.len() {
            // We make room for several extra entity ids to avoid
            // increasing the capacity by 1 over and over
            // and thus causing lots of copying.
            self.generations.resize(entity_id + 10, 0);
        }
        dbg!(entity_id, self.generations.len());
        debug_assert!(entity_id < self.generations.len());
        self.generations[entity_id]
    }
}

struct ComponentPool<T> {
    components: Vec<Option<T>>,
}

impl<T> ComponentPool<T> {}

struct Registry {
    /// The maximum entity id we have issued. This is the "length" of the Registry.
    max_entity_id: usize,
    /// The maximum number of entities we are capable of issuing.
    /// This will need to grow when max_entity_id becomes large enough.
    entity_capacity: usize,
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
            max_entity_id: 0,
            entity_capacity: 100,
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
            id: self.max_entity_id,
            generation: 0,
        };
        self.max_entity_id += 1;
        new_entity
    }
}

#[test]
fn test_entity_generations_happy_path() {
    let mut eg = EntityGenerations::new();
    assert_eq!(eg.get(0), 0);
    assert_eq!(eg.get(1), 0);
    assert_eq!(eg.get(10), 0);

    let mut eg = EntityGenerations::new();
    assert_eq!(eg.get(10), 0);
}

#[test]
fn test_registry_happy_path() {
    let mut registry: Registry = Registry::new();
    registry.create_entity();
}
