//! Provides the plugin infrastructure.

use event::{Event, EventHandler, Dispatcher, SendElement, Priority};

use std::any::Any;

use std::sync::{Arc, Mutex};

use std::mem;

use minidom::Element;

#[derive(Clone)]
pub struct PluginProxyBinding {
    dispatcher: Arc<Mutex<Dispatcher>>,
}

impl PluginProxyBinding {
    pub fn new(dispatcher: Arc<Mutex<Dispatcher>>) -> PluginProxyBinding {
        PluginProxyBinding {
            dispatcher: dispatcher,
        }
    }
}

pub enum PluginProxy {
    Unbound,
    BoundTo(PluginProxyBinding),
}

impl PluginProxy {
    /// Returns a new `PluginProxy`.
    pub fn new() -> PluginProxy {
        PluginProxy::Unbound
    }

    /// Binds the `PluginProxy` to a `PluginProxyBinding`.
    pub fn bind(&mut self, inner: PluginProxyBinding) {
        if let PluginProxy::BoundTo(_) = *self {
            panic!("trying to bind an already bound plugin proxy!");
        }
        mem::replace(self, PluginProxy::BoundTo(inner));
    }

    fn with_binding<R, F: FnOnce(&PluginProxyBinding) -> R>(&self, f: F) -> R {
        match *self {
            PluginProxy::Unbound => {
                panic!("trying to use an unbound plugin proxy!");
            },
            PluginProxy::BoundTo(ref binding) => {
                f(binding)
            },
        }
    }

    /// Dispatches an event.
    pub fn dispatch<E: Event>(&self, event: E) {
        self.with_binding(move |binding| {
            // TODO: proper error handling
            binding.dispatcher.lock().unwrap().dispatch(event);
        });
    }

    /// Registers an event handler.
    pub fn register_handler<E, H>(&self, priority: Priority, handler: H) where E: Event, H: EventHandler<E> {
        self.with_binding(move |binding| {
            // TODO: proper error handling
            binding.dispatcher.lock().unwrap().register(priority, handler);
        });
    }

    /// Sends a stanza.
    pub fn send(&self, elem: Element) {
        self.dispatch(SendElement(elem));
    }
}

/// A trait whch all plugins should implement.
pub trait Plugin: Any + PluginAny {
    /// Gets a mutable reference to the inner `PluginProxy`.
    fn get_proxy(&mut self) -> &mut PluginProxy;

    #[doc(hidden)]
    fn bind(&mut self, inner: PluginProxyBinding) {
        self.get_proxy().bind(inner);
    }
}

pub trait PluginInit {
    fn init(dispatcher: &mut Dispatcher, me: Arc<Box<Plugin>>);
}

pub trait PluginAny {
    fn as_any(&self) -> &Any;
}

impl<T: Any + Sized + Plugin> PluginAny for T {
    fn as_any(&self) -> &Any { self }
}
