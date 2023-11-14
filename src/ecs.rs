struct ComponentPool<T> {
    components: Vec<Option<T>>,
}

impl<T> ComponentPool<T> {}

struct Registry {
    component_pools: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl Registry {}
