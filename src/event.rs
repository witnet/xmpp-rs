use std::marker::PhantomData;
use std::any::{TypeId, Any};
use std::fmt::Debug;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::mem;

use minidom::Element;

/// A marker trait which marks all events.
pub trait Event: Any + Debug {}

/// A trait which is implemented for all event handlers.
trait EventHandler: Any {
    /// Handle an event, returns whether to propagate the event to the remaining handlers.
    fn handle(&self, event: &AbstractEvent) -> Propagation;
}

/// An abstract event.
pub struct AbstractEvent {
    inner: Box<Any>,
}

impl AbstractEvent {
    /// Creates an abstract event from a concrete event.
    pub fn new<E: Event>(event: E) -> AbstractEvent {
        AbstractEvent {
            inner: Box::new(event),
        }
    }

    /// Downcasts this abstract event into a concrete event.
    pub fn downcast<E: Event + 'static>(&self) -> Option<&E> {
        self.inner.downcast_ref::<E>()
    }

    /// Checks whether this abstract event is a specific concrete event.
    pub fn is<E: Event + 'static>(&self) -> bool {
        self.inner.is::<E>()
    }
}

struct Record<P, T>(P, T);

impl<P: PartialEq, T> PartialEq for Record<P, T> {
    fn eq(&self, other: &Record<P, T>) -> bool {
        self.0 == other.0
    }
}

impl<P: Eq, T> Eq for Record<P, T> {}

impl<P: PartialOrd, T> PartialOrd for Record<P, T> {
    fn partial_cmp(&self, other: &Record<P, T>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<P: Ord, T> Ord for Record<P, T> {
    fn cmp(&self, other: &Record<P, T>) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// An enum representing whether to keep propagating an event or to stop the propagation.
pub enum Propagation {
    /// Stop the propagation of the event, the remaining handlers will not get invoked.
    Stop,
    /// Continue propagating the event.
    Continue,
}

/// An event dispatcher, this takes care of dispatching events to their respective handlers.
pub struct Dispatcher {
    handlers: BTreeMap<TypeId, Vec<Record<Priority, Box<EventHandler>>>>,
    queue: Vec<(TypeId, AbstractEvent)>,
}

impl Dispatcher {
    /// Create a new `Dispatcher`.
    pub fn new() -> Dispatcher {
        Dispatcher {
            handlers: BTreeMap::new(),
            queue: Vec::new(),
        }
    }

    /// Register an event handler.
    pub fn register<E, F>(&mut self, priority: Priority, func: F)
        where
            E: Event,
            F: Fn(&E) -> Propagation + 'static {
        struct Handler<E, F> where E: Event, F: Fn(&E) -> Propagation {
            func: F,
            _marker: PhantomData<E>,
        }

        impl<E: Event, F: Fn(&E) -> Propagation + 'static> EventHandler for Handler<E, F> {
            fn handle(&self, evt: &AbstractEvent) -> Propagation {
                if let Some(e) = evt.downcast::<E>() {
                    (self.func)(e)
                }
                else {
                    Propagation::Continue
                }
            }
        }

        let handler: Box<EventHandler> = Box::new(Handler {
            func: func,
            _marker: PhantomData,
        }) as Box<EventHandler>;
        let ent = self.handlers.entry(TypeId::of::<E>())
                               .or_insert_with(|| Vec::new());
        ent.push(Record(priority, handler));
        ent.sort();
    }

    /// Append an event to the queue.
    pub fn dispatch<E>(&mut self, event: E) where E: Event {
        self.queue.push((TypeId::of::<E>(), AbstractEvent::new(event)));
    }

    /// Flush all events in the queue so they can be handled by their respective handlers.
    /// Returns whether there are still pending events.
    pub fn flush(&mut self) -> bool {
        let mut q = Vec::new();
        mem::swap(&mut self.queue, &mut q);
        'evts: for (t, evt) in q {
            if let Some(handlers) = self.handlers.get_mut(&t) {
                for &mut Record(_, ref mut handler) in handlers {
                    match handler.handle(&evt) {
                        Propagation::Stop => { continue 'evts; },
                        Propagation::Continue => (),
                    }
                }
            }
        }
        !self.queue.is_empty()
    }

    /// Flushes all events, like `flush`, but keeps doing this until there is nothing left in the
    /// queue.
    pub fn flush_all(&mut self) {
        while self.flush() {}
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Max,
    Default,
    Min,
}

impl Default for Priority {
    fn default() -> Priority {
        Priority::Default
    }
}

#[derive(Debug)]
pub struct SendElement(pub Element);

impl Event for SendElement {}

#[derive(Debug)]
pub struct ReceiveElement(pub Element);

impl Event for ReceiveElement {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "success")]
    fn test() {
        let mut disp = Dispatcher::new();

        #[derive(Debug)]
        struct MyEvent {
            should_be_42: u32,
        }

        impl Event for MyEvent {}

        disp.register(Priority::Max, |evt: &MyEvent| {
            if evt.should_be_42 == 42 {
                Propagation::Continue
            }
            else {
                Propagation::Stop
            }
        });

        disp.register(Priority::Min, |_: &MyEvent| {
            panic!("should not be called");
        });

        disp.register(Priority::Default, |evt: &MyEvent| {
            if evt.should_be_42 == 42 {
                panic!("success");
            }
            else {
                panic!("not 42");
            }
        });

        disp.register(Priority::Min, |_: &MyEvent| {
            panic!("should not be called");
        });

        disp.dispatch(MyEvent {
            should_be_42: 39,
        });

        disp.dispatch(MyEvent {
            should_be_42: 42,
        });

        disp.flush();
    }
}
