use std::fmt::Debug;

use std::any::Any;

pub struct AbstractEvent {
    inner: Box<Any>,
}

impl AbstractEvent {
    pub fn new<E: Event>(event: E) -> AbstractEvent {
        AbstractEvent {
            inner: Box::new(event),
        }
    }

    pub fn downcast<E: Event + 'static>(&self) -> Option<&E> {
        self.inner.downcast_ref::<E>()
    }

    pub fn is<E: Event + 'static>(&self) -> bool {
        self.inner.is::<E>()
    }
}

pub trait Event: Any + Debug {}
