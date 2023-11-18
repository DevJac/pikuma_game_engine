#[derive(Debug)]
enum DeadEntity {
    DeadEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
struct Entity {
    id: usize,
    generation: usize,
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.generation.cmp(&other.generation))
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

struct SystemInput {
    entities: std::collections::BTreeSet<Entity>,
}

impl SystemInput {
    fn new() -> Self {
        Self {
            entities: std::collections::BTreeSet::new(),
        }
    }

    fn get_entities(&self) -> Vec<&Entity> {
        self.entities.iter().collect()
    }
}

trait System {
    fn as_any(&self) -> &dyn std::any::Any;
    fn required_components(&self) -> Vec<std::any::TypeId>;
    fn add_entity(&mut self, entity: Entity);
    fn remove_entity(&mut self, entit: Entity);
    fn run(&self, registry: &mut Registry);
}
// TODO: required_components function
// TODO: matching entities function
// TODO: run function
// a system is just a function that receives / queries certain components and alters them
// a system might store a list of entities that contain the required components
// what's wrong with just iterating every entity every time?
// that would require iterating a lot of irrelevant entities
// we can avoid iterating irrelevant entities by storing entities we know are relevant
// we can iterate the stored relevant entities in our system code
// this works for a struct with static / unchanging methods / code
// how can we have user customizable code that still has access to the list of relevant entities?
// what do we do with component pools?
// component pools are different, they store only generic data, just data, no generic code
// code is just data, at least in some languages, is Rust dynamic enough to treat code as just data?
// if the system struct stores a Fn, can it call the Fn and pass itself to the Fn?
// probably, I don't see why not, a impl method would have a mutable share of self,
// so it could pass self to sub-functions
// self could be designed to expose some nice APIs that the Fn could use
// what <T> would the Fn accept though?
// what is <T> in this System?
// When the system code calls getEntities, what entities will be returned? That is T.
// sometimes we'll want one component, sometimes more,
// I don't think a variable number of components can fit in a single generic T
// We may have to make System a trait
// what would we need in a system trait?
// TODO: required_components function to help the trait sort and assign relevant entities
// TODO: add_relevant_entity function
// TODO: remove_relevant_entity function
// TODO: get_relevant_entities function to be used by the run function
// TODO: run function to run the system code
// A type encompasas many values, just data
// A Fn encompases many values as well, code instead of data
// To encompass many values / data and code we have to use a trait
// Our systems will have to have a common trait that exposes the required values / data and code

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
    systems: std::collections::HashMap<std::any::TypeId, Box<dyn System>>,
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

    fn run_systems(&mut self) {
        let mut systems = Vec::new();
        for system in self.systems.values() {
            let a = system.as_any().downcast_ref::<&dyn System>().unwrap();
            systems.push(a);
        }
        for system in systems {
            // We can't give an exclusive borrow to the whole struct, because the systems are being iterated.
            // Our systems would not mutate the systems though, only the components.
            // We should store the components in a different field than the systems, then we can lend the components
            // while iterating the systems.
            system.run(self);
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

    struct CounterIncrementSystem {
        entities: std::collections::BTreeSet<Entity>,
    }

    impl CounterIncrementSystem {
        fn new() -> Self {
            Self {
                entities: std::collections::BTreeSet::new(),
            }
        }

        fn get_entities(&self) -> Vec<&Entity> {
            self.entities.iter().collect()
        }
    }

    impl System for CounterIncrementSystem {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn required_components(&self) -> Vec<std::any::TypeId> {
            vec![std::any::TypeId::of::<CounterComponent>()]
        }

        fn add_entity(&mut self, entity: Entity) {
            self.entities.insert(entity);
        }

        fn remove_entity(&mut self, entity: Entity) {
            self.entities.remove(&entity);
        }

        fn run(&self, registry: &mut Registry) {
            for entity in self.get_entities() {
                let counter_component: &mut CounterComponent =
                    registry.get_component_mut(*entity).unwrap().unwrap();
                counter_component.count += 1;
            }
        }
    }

    let mut registry = Registry::new();
    let e = registry.create_entity();
    registry
        .add_component(e, CounterComponent { count: 0 })
        .unwrap();
    registry.add_system(CounterIncrementSystem::new());
}
