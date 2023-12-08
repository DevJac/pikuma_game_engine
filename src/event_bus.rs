use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait HandlerBase {
    fn handle_any(&mut self, event: &dyn Any);
}

pub trait Handler<E>: HandlerBase {
    fn handle(&mut self, event: E);
}

pub struct EventBus {
    handlers: HashMap<TypeId, Vec<Box<dyn HandlerBase>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler<E: 'static, H: Handler<E> + 'static>(&mut self, handler: H) {
        let type_id = TypeId::of::<E>();
        match self.handlers.get_mut(&type_id) {
            None => {
                self.handlers.insert(type_id, vec![Box::new(handler)]);
            }
            Some(handlers) => {
                handlers.push(Box::new(handler));
            }
        }
    }

    pub fn dispatch<E: 'static>(&mut self, event: &'static E) {
        let type_id = TypeId::of::<E>();
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for handler in handlers {
                handler.handle_any(&event);
            }
        }
    }
}
