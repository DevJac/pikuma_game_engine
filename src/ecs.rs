use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

type IndexT = u32;
type GenerationT = u32;

const VEC_RESIZE_MARGIN: usize = 10;

#[derive(Debug)]
pub enum EcsError {
    DeadEntity,
    NoSuchComponent,
    NoSuchSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct Entity {
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
    entity_components: HashMap<Entity, HashSet<TypeId>>,
    component_pools: HashMap<TypeId, Box<dyn Any>>,
}

impl EntityComponentManager {
    fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            entity_components: HashMap::new(),
            component_pools: HashMap::new(),
        }
    }

    fn create_entity(&mut self) -> Entity {
        let new_entity = self.entity_manager.create_entity();
        self.entity_components.insert(new_entity, HashSet::new());
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
        if self.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: TypeId = TypeId::of::<T>();
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
        if self.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: TypeId = TypeId::of::<T>();
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
        if self.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: TypeId = TypeId::of::<T>();
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
        if self.is_dead(entity) {
            return Err(EcsError::DeadEntity);
        }
        let type_id: TypeId = TypeId::of::<T>();
        match self.component_pools.get_mut(&type_id) {
            None => Err(EcsError::NoSuchComponent),
            Some(component_pool) => {
                let component_pool: &mut ComponentPool<T> =
                    (&mut **component_pool).downcast_mut().unwrap();
                Ok(component_pool.get_mut(entity))
            }
        }
    }

    fn has_components(&self, entity: Entity) -> &HashSet<TypeId> {
        self.entity_components.get(&entity).unwrap()
    }

    fn entities_and_components(&self) -> impl Iterator<Item = (&Entity, &HashSet<TypeId>)> {
        self.entity_components.iter()
    }
}

pub struct EntityComponentWrapper<'ec> {
    ec_manager: &'ec mut EntityComponentManager,
    changed_entities: HashSet<Entity>,
}

impl<'ec> EntityComponentWrapper<'ec> {
    fn new(ec_manager: &'ec mut EntityComponentManager) -> Self {
        Self {
            ec_manager,
            changed_entities: HashSet::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let new_entity = self.ec_manager.create_entity();
        self.changed_entities.insert(new_entity);
        new_entity
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
        self.changed_entities.insert(entity);
        self.ec_manager.remove_entity(entity)
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.ec_manager.is_alive(entity)
    }

    pub fn is_dead(&self, entity: Entity) -> bool {
        self.ec_manager.is_dead(entity)
    }

    pub fn add_component<T: Clone + 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        self.changed_entities.insert(entity);
        self.ec_manager.add_component(entity, component)
    }

    pub fn remove_component<T: Clone + 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        self.changed_entities.insert(entity);
        self.ec_manager.remove_component::<T>(entity)
    }

    pub fn get_component<T: Clone + 'static>(
        &self,
        entity: Entity,
    ) -> Result<Option<&T>, EcsError> {
        self.ec_manager.get_component(entity)
    }

    pub fn get_component_mut<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&mut T>, EcsError> {
        self.ec_manager.get_component_mut(entity)
    }

    pub fn has_components(&self, entity: Entity) -> &HashSet<TypeId> {
        self.ec_manager.has_components(entity)
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.ec_manager.entities_and_components().map(|(e, _c)| e)
    }

    pub fn entities_and_components(&self) -> impl Iterator<Item = (&Entity, &HashSet<TypeId>)> {
        self.ec_manager.entities_and_components()
    }

    pub fn changed_entities(&self) -> impl Iterator<Item = &Entity> {
        self.changed_entities.iter()
    }
}

pub trait SystemBase {
    fn as_any(&self) -> &dyn Any;
    fn required_components(&self) -> &HashSet<TypeId>;
    fn add_entity(&mut self, entity: Entity);
    fn remove_entity(&mut self, entity: Entity);
}

pub trait System: SystemBase {
    type Input<'i>;
    fn run<'i>(&self, ec_manager: &mut EntityComponentWrapper, input: Self::Input<'i>);
}

pub struct Registry {
    ec_manager: EntityComponentManager,
    systems: HashMap<TypeId, Box<dyn SystemBase>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            ec_manager: EntityComponentManager::new(),
            systems: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        // Because a new entity has no components, no systems will be interested in it.
        self.ec_manager.create_entity()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), EcsError> {
        for system in self.systems.values_mut() {
            system.remove_entity(entity);
        }
        self.ec_manager.remove_entity(entity)
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.ec_manager.is_alive(entity)
    }

    pub fn is_dead(&self, entity: Entity) -> bool {
        self.ec_manager.is_dead(entity)
    }

    pub fn add_component<T: Clone + 'static>(
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

    pub fn remove_component<T: Clone + 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        let result = self.ec_manager.remove_component::<T>(entity);
        if result.is_ok() {
            for system in self.systems.values_mut() {
                if !self
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

    pub fn get_component<T: Clone + 'static>(
        &self,
        entity: Entity,
    ) -> Result<Option<&T>, EcsError> {
        self.ec_manager.get_component(entity)
    }

    pub fn get_component_mut<T: Clone + 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<Option<&mut T>, EcsError> {
        self.ec_manager.get_component_mut(entity)
    }

    pub fn add_system<S: System + 'static>(&mut self, mut system: S) {
        for (entity, components) in self.ec_manager.entities_and_components() {
            if components.is_superset(system.required_components()) {
                system.add_entity(*entity);
            }
        }
        let type_id: TypeId = TypeId::of::<S>();
        self.systems.insert(type_id, Box::new(system));
    }

    pub fn remove_system<S: System + 'static>(&mut self) {
        let type_id: TypeId = TypeId::of::<S>();
        self.systems.remove(&type_id);
    }

    fn get_system<S: System + 'static>(
        systems: &HashMap<TypeId, Box<dyn SystemBase>>,
    ) -> Option<&S> {
        let type_id = TypeId::of::<S>();
        if let Some(system_any) = systems.get(&type_id) {
            return system_any.as_any().downcast_ref::<S>();
        }
        None
    }

    pub fn run_system<S: System + 'static>(&mut self, input: S::Input<'_>) -> Result<(), EcsError> {
        let mut ec_wrapper = EntityComponentWrapper::new(&mut self.ec_manager);
        let system = Self::get_system::<S>(&self.systems);
        if system.is_none() {
            return Err(EcsError::NoSuchSystem);
        }
        system.unwrap().run(&mut ec_wrapper, input);
        for entity in ec_wrapper.changed_entities() {
            for system in self.systems.values_mut() {
                if ec_wrapper
                    .has_components(*entity)
                    .is_superset(system.required_components())
                {
                    system.add_entity(*entity);
                } else {
                    system.remove_entity(*entity);
                }
            }
        }
        Ok(())
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.ec_manager.entities_and_components().map(|(e, _c)| e)
    }

    pub fn entities_and_components(&self) -> impl Iterator<Item = (&Entity, &HashSet<TypeId>)> {
        self.ec_manager.entities_and_components()
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityComponentWrapper, EntityManager, Registry, System, SystemBase};
    use std::any::{Any, TypeId};
    use std::collections::HashSet;

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
        required_components: HashSet<TypeId>,
        entities: HashSet<Entity>,
        expected_entity_count: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    impl CounterIncrementSystem {
        fn new() -> Self {
            let mut required_components = HashSet::new();
            required_components.insert(TypeId::of::<CounterComponent>());
            Self {
                required_components,
                entities: HashSet::new(),
                expected_entity_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    impl SystemBase for CounterIncrementSystem {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn required_components(&self) -> &HashSet<TypeId> {
            &self.required_components
        }

        fn add_entity(&mut self, entity: Entity) {
            self.entities.insert(entity);
        }

        fn remove_entity(&mut self, entity: Entity) {
            self.entities.remove(&entity);
        }
    }

    impl System for CounterIncrementSystem {
        type Input<'i> = u32;

        fn run(&self, ec_manager: &mut EntityComponentWrapper, increment_amount: Self::Input<'_>) {
            assert_eq!(
                self.entities.len(),
                *self.expected_entity_count.lock().unwrap()
            );
            for entity in self.entities.iter() {
                let counter_component: &mut CounterComponent =
                    ec_manager.get_component_mut(*entity).unwrap().unwrap();
                counter_component.count += increment_amount;
            }
            let e = ec_manager.create_entity();
            ec_manager
                .add_component(e, CounterComponent { count: 0 })
                .unwrap();
        }
    }

    #[test]
    fn test_system_happy_path() {
        let mut registry = Registry::new();
        let e = registry.create_entity();
        let system = CounterIncrementSystem::new();
        let expected_entity_count = system.expected_entity_count.clone();
        registry
            .add_component(e, CounterComponent { count: 0 })
            .unwrap();
        registry.add_system(system);
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            0
        );
        assert_eq!(registry.entities().count(), 1);
        *expected_entity_count.lock().unwrap() = 1;
        registry.run_system::<CounterIncrementSystem>(1).unwrap();
        assert_eq!(registry.entities().count(), 2);
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            1
        );
        assert_eq!(registry.entities().count(), 2);
        *expected_entity_count.lock().unwrap() = 2;
        registry.run_system::<CounterIncrementSystem>(1).unwrap();
        assert_eq!(registry.entities().count(), 3);
        assert_eq!(
            registry
                .get_component::<CounterComponent>(e)
                .unwrap()
                .unwrap()
                .count,
            2
        );

        registry.remove_component::<CounterComponent>(e).unwrap();
        assert_eq!(registry.entities().count(), 3);
        *expected_entity_count.lock().unwrap() = 2;
        registry.run_system::<CounterIncrementSystem>(1).unwrap();
        assert_eq!(registry.entities().count(), 4);

        registry.remove_entity(e).unwrap();
        assert_eq!(registry.entities().count(), 3);
        *expected_entity_count.lock().unwrap() = 3;
        registry.run_system::<CounterIncrementSystem>(1).unwrap();
        assert_eq!(registry.entities().count(), 4);
    }
}
