use std::marker::PhantomData;
use std::any::{TypeId, Any};
use std::fmt::Debug;
use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::sync::Arc;
use std::mem;
use std::ptr;
use std::raw::TraitObject;

use minidom::Element;

/// A marker trait which marks all events.
pub trait Event: Any + Debug {}

/// A trait which can be implemented when something can handle a specific kind of event.
pub trait EventHandler<E: Event>: Any {
    /// Handle an event, returns whether to propagate the event to the remaining handlers.
    fn handle(&self, event: &E) -> Propagation;
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

#[derive(Debug)]
struct GarbageEvent;

impl Event for GarbageEvent {}

impl<E, F> EventHandler<E> for Box<F> where E: Event, F: 'static + Fn(&E) -> Propagation {
    fn handle(&self, evt: &E) -> Propagation {
        self(evt)
    }
}

/// An event dispatcher, this takes care of dispatching events to their respective handlers.
pub struct Dispatcher {
    handlers: BTreeMap<TypeId, Vec<Record<Priority, Box<Any>>>>,
    queue: Vec<(TypeId, Box<Any>)>,
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
    pub fn register<E, H>(&mut self, priority: Priority, handler: H) where E: Event + 'static, H: EventHandler<E> {
        let handler: Box<EventHandler<E>> = Box::new(handler) as Box<EventHandler<E>>;
        let ent = self.handlers.entry(TypeId::of::<E>())
                               .or_insert_with(|| Vec::new());
        ent.push(Record(priority, Box::new(handler) as Box<Any>));
        ent.sort();
    }

    /// Append an event to the queue.
    pub fn dispatch<E>(&mut self, event: E) where E: Event {
        self.queue.push((TypeId::of::<E>(), Box::new(event) as Box<Any>));
    }

    /// Flush all events in the queue so they can be handled by their respective handlers.
    /// Returns whether there are still pending events.
    pub fn flush(&mut self) -> bool {
        let mut q = Vec::new();
        mem::swap(&mut self.queue, &mut q);
        'evts: for (t, evt) in q {
            if let Some(handlers) = self.handlers.get_mut(&t) {
                for &mut Record(_, ref mut handler) in handlers {
                    // GarbageEvent is a garbage type.
                    // The actual passed type is NEVER of this type.
                    let h: &mut EventHandler<GarbageEvent> = unsafe {
                        let handler_obj: &mut TraitObject = mem::transmute(handler);
                        let handler_inner: *mut TraitObject = mem::transmute(handler_obj.data);
                        mem::transmute(*handler_inner)
                    };
                    let e: &&GarbageEvent = unsafe {
                        let evt_ref: &Any = &evt;
                        let evt_obj: TraitObject = mem::transmute(evt_ref);
                        mem::transmute(evt_obj.data)
                    };
                    match h.handle(e) {
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

    /// Dispatch an event to the handlers right now, without going through the queue.
    pub fn dispatch_now<E>(&mut self, event: E) where E: Event {
        if let Some(handlers) = self.handlers.get_mut(&TypeId::of::<E>()) {
            for &mut Record(_, ref mut handler) in handlers {
                let h = handler.downcast_mut::<Box<EventHandler<E>>>().unwrap();
                match h.handle(&event) {
                    Propagation::Stop => { return; },
                    Propagation::Continue => (),
                }
            }
        }
    }
}

pub struct EventProxy<T: ?Sized, E: Event> {
    inner: Arc<Box<T>>,
    vtable: *mut (),
    _event_type: PhantomData<E>,
}

impl<T: ?Sized, E: Event> EventProxy<T, E> {
    /// Unsafe because T is assumed to be a TraitObject or at least have its shape.
    /// If it is not, things will break. In a fascinatingly horrible manner.
    /// Some people, such as myself, find it hilarious. Most people do not.
    /// T is also assumed to actually support EventHandler<E>, if it does not, refer to above
    /// statement.
    pub unsafe fn new<H: EventHandler<E>>(inner: Arc<Box<T>>) -> EventProxy<T, E> {
        let box_with_vtable = &*ptr::null::<H>() as &EventHandler<E>;
        let obj: TraitObject = mem::transmute(box_with_vtable);
        EventProxy {
            inner: inner,
            vtable: obj.vtable,
            _event_type: PhantomData,
        }
    }
}

impl<T: ?Sized, E: Event> EventHandler<E> for EventProxy<T, E> where Box<T>: 'static {
    fn handle(&self, evt: &E) -> Propagation {
        let inner = Arc::into_raw(self.inner.clone());
        let obj = TraitObject { data: unsafe { mem::transmute(inner) }, vtable: self.vtable };
        let handler: &EventHandler<E> = unsafe { mem::transmute(obj) };
        let prop = handler.handle(evt);
        unsafe { Arc::<Box<T>>::from_raw(mem::transmute(inner)); }
        prop
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

        struct MyHandler;
        struct EvilHandler;
        struct EventFilter;

        #[derive(Debug)]
        struct MyEvent {
            should_be_42: u32,
        }

        impl Event for MyEvent {}

        impl EventHandler<MyEvent> for MyHandler {
            fn handle(&self, evt: &MyEvent) -> Propagation {
                if evt.should_be_42 == 42 {
                    panic!("success");
                }
                else {
                    panic!("not 42");
                }
            }
        }

        impl EventHandler<MyEvent> for EvilHandler {
            fn handle(&self, _: &MyEvent) -> Propagation {
                panic!("should not be called");
            }
        }

        impl EventHandler<MyEvent> for EventFilter {
            fn handle(&self, evt: &MyEvent) -> Propagation {
                if evt.should_be_42 == 42 {
                    Propagation::Continue
                }
                else {
                    Propagation::Stop
                }
            }
        }

        disp.register(Priority::Max, EventFilter);
        disp.register(Priority::Min, EvilHandler);
        disp.register(Priority::Default, MyHandler);
        disp.register(Priority::Min, EvilHandler);

        disp.dispatch(MyEvent {
            should_be_42: 39,
        });

        disp.dispatch(MyEvent {
            should_be_42: 42,
        });

        disp.flush();
    }
}
