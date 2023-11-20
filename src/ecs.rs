type IndexT = u32;
type GenerationT = u32;

const VEC_RESIZE_MARGIN: usize = 10;

#[derive(Debug)]
enum EcsError {
    DeadEntity,
    NoSuchComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
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
    component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl EntityComponentManager {
    fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            component_pools: std::collections::HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        self.entity_manager.create_entity()
    }

    fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
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
}

trait System {
    fn required_components(&self) -> Vec<std::any::TypeId>;
    fn update_entity(&mut self, entity: Entity, ec_manager: EntityComponentManager);
    fn run(&self, ec_manager: EntityComponentManager);
}

struct SystemManager {
    entity_components: EntityComponentManager,
    systems: std::collections::HashMap<std::any::TypeId, Box<dyn System>>,
}

impl SystemManager {
    fn new() -> Self {
        todo!()
    }

    fn add_system<T: System>(&mut self, system: T) {
        todo!()
    }

    fn remove_system<T: System>(&mut self) {
        todo!()
    }

    fn update_entity(&mut self, entity: Entity, ec_manager: EntityComponentManager) {
        todo!()
    }
}

struct Registry {}

impl Registry {}

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
    assert!(!em.is_alive(e1));
    assert!(em.is_alive(e2));

    assert!(em.remove_entity(e1).is_err());

    let e3: Entity = em.create_entity();
    assert_eq!(e3.id, 1);
    assert!(em.is_alive(e0));
    assert!(!em.is_alive(e1));
    assert!(em.is_alive(e2));
    assert!(em.is_alive(e3));

    assert!(em.remove_entity(e1).is_err());
}

//
// #[test]
// fn test_registry_happy_path() {
//     let mut registry: Registry = Registry::new();
//     let e0: Entity = registry.create_entity();
//     let e1: Entity = registry.create_entity();
//
//     let e2: Entity = registry.create_entity();
//     registry.add_component(e2, 5_i32).unwrap();
//     assert_eq!(registry.get_component::<i32>(e2).unwrap().unwrap(), &5_i32);
//     registry.remove_entity(e2).unwrap();
//     let e2: Entity = registry.create_entity();
//     assert_eq!(registry.get_component::<i32>(e2).unwrap(), None);
//
//     assert_eq!(registry.next_entity_id, 3);
//     registry.remove_entity(e0).unwrap();
//     registry.remove_entity(e1).unwrap();
//     assert_eq!(registry.next_entity_id, 3);
//     let _e0: Entity = registry.create_entity();
//     let _e1: Entity = registry.create_entity();
//     assert_eq!(registry.next_entity_id, 3);
//     let _e3: Entity = registry.create_entity();
//     assert_eq!(registry.next_entity_id, 4);
//
//     registry.add_component(e2, 5_i32).unwrap();
//     registry.remove_entity(e2).unwrap();
//     assert!(registry.add_component(e2, 5_i32).is_err());
// }
//
// struct Registry {
//     /// The maximum entity id we have issued. This is the "length" of the Registry.
//     next_entity_id: usize,
//     /// Entity ids that are free to issue again.
//     free_entity_ids: Vec<usize>,
//     /// The current generation of the entities.
//     /// If a given Entity has a generation less than this,
//     /// that Entity is no longer valid.
//     entity_generations: EntityGenerations,
//     /// The ComponentPools / Vectors that store components / values.
//     component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
//     systems: std::collections::HashMap<std::any::TypeId, Box<dyn System>>,
// }
//
// impl Registry {
//     fn new() -> Self {
//         Self {
//             next_entity_id: 0,
//             free_entity_ids: Vec::new(),
//             entity_generations: EntityGenerations::new(),
//             component_pools: std::collections::HashMap::new(),
//             systems: std::collections::HashMap::new(),
//         }
//     }
//
//     fn create_entity(&mut self) -> Entity {
//         if let Some(entity_id) = self.free_entity_ids.pop() {
//             return Entity {
//                 id: entity_id,
//                 generation: self.entity_generations.get(entity_id),
//             };
//         }
//         let new_entity = Entity {
//             id: self.next_entity_id,
//             generation: 0,
//         };
//         self.next_entity_id += 1;
//         new_entity
//     }
//
//     fn is_entity_alive(&self, entity: Entity) -> bool {
//         self.entity_generations.is_alive(entity)
//     }
//
//     fn remove_entity(&mut self, entity: Entity) -> Result<(), DeadEntity> {
//         if !self.is_entity_alive(entity) {
//             return Err(DeadEntity::DeadEntity);
//         }
//         self.entity_generations.increment(entity.id);
//         self.free_entity_ids.push(entity.id);
//         Ok(())
//     }
//
//     fn add_component<T: Clone + 'static>(
//         &mut self,
//         entity: Entity,
//         component: T,
//     ) -> Result<(), DeadEntity> {
//         if !self.is_entity_alive(entity) {
//             return Err(DeadEntity::DeadEntity);
//         }
//         let type_id = std::any::TypeId::of::<T>();
//         match self.component_pools.get_mut(&type_id) {
//             None => {
//                 let new_component_pool = Box::new(ComponentPool::<T>::new_one(entity, component));
//                 self.component_pools.insert(type_id, new_component_pool);
//             }
//             Some(component_pool) => {
//                 let component_pool: &mut ComponentPool<T> =
//                     (&mut **component_pool).downcast_mut().unwrap();
//                 component_pool.set(entity, component);
//             }
//         }
//         Ok(())
//     }
//
//     fn get_component<T: Clone + 'static>(&self, entity: Entity) -> Result<Option<&T>, DeadEntity> {
//         if !self.is_entity_alive(entity) {
//             return Err(DeadEntity::DeadEntity);
//         }
//         let type_id = std::any::TypeId::of::<T>();
//         match self.component_pools.get(&type_id) {
//             None => {
//                 return Ok(None);
//             }
//             Some(component_pool) => {
//                 let component_pool: &ComponentPool<T> = (&**component_pool).downcast_ref().unwrap();
//                 return Ok(component_pool.get(entity));
//             }
//         }
//     }
//
//     fn get_component_mut<T: Clone + 'static>(
//         &mut self,
//         entity: Entity,
//     ) -> Result<Option<&mut T>, DeadEntity> {
//         if !self.is_entity_alive(entity) {
//             return Err(DeadEntity::DeadEntity);
//         }
//         let type_id = std::any::TypeId::of::<T>();
//         match self.component_pools.get_mut(&type_id) {
//             None => {
//                 return Ok(None);
//             }
//             Some(component_pool) => {
//                 let component_pool: &mut ComponentPool<T> =
//                     (&mut **component_pool).downcast_mut().unwrap();
//                 return Ok(component_pool.get_mut(entity));
//             }
//         }
//     }
//
//     fn add_system<T: System + 'static>(&mut self, system: T) {
//         let type_id = std::any::TypeId::of::<T>();
//         self.systems.insert(type_id, Box::new(system));
//     }
//
//     fn run_systems(&mut self) {
//         let mut systems = Vec::new();
//         for system in self.systems.values() {
//             let a = system.as_any().downcast_ref::<&dyn System>().unwrap();
//             systems.push(a);
//         }
//         for system in systems {
//             // We can't give an exclusive borrow to the whole struct, because the systems are being iterated.
//             // Our systems would not mutate the systems though, only the components.
//             // We should store the components in a different field than the systems, then we can lend the components
//             // while iterating the systems.
//             system.run(self);
//         }
//     }
// }
