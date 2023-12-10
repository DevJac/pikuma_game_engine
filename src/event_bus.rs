use crate::ecs::EntityComponentWrapper;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait HandlerBase {
    fn handle_any(&mut self, ec_manager: &mut EntityComponentWrapper, event: &dyn Any);
}

pub trait Handler<E>: HandlerBase {
    fn handle(&mut self, ec_manager: &mut EntityComponentWrapper, event: &E);
}

pub struct EventBus {
    handlers: HashMap<TypeId, Vec<Rc<RefCell<dyn HandlerBase>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler<E: 'static, H: Handler<E> + 'static>(&mut self, handler: Rc<RefCell<H>>) {
        let type_id = TypeId::of::<E>();
        match self.handlers.get_mut(&type_id) {
            None => {
                self.handlers.insert(type_id, vec![handler]);
            }
            Some(handlers) => {
                handlers.push(handler);
            }
        }
    }

    pub fn dispatch(
        &mut self,
        ec_manager: &mut EntityComponentWrapper,
        type_id: TypeId,
        event: &dyn Any,
    ) {
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for handler in handlers {
                handler.borrow_mut().handle_any(ec_manager, event);
            }
        } else {
        }
    }
}
