//! Provides the plugin infrastructure.

use event::{Event, Dispatcher, SendElement, Priority, Propagation};

use std::any::{Any, TypeId};

use std::collections::HashMap;

use std::sync::{RwLock, Arc};

use std::marker::PhantomData;

use std::ops::Deref;

use std::convert::AsRef;

use std::mem;

use minidom::Element;

pub struct PluginContainer {
    plugins: RwLock<HashMap<TypeId, Arc<Plugin>>>,
}

impl PluginContainer {
    pub fn new() -> PluginContainer {
        PluginContainer {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<P: Plugin + 'static>(&self, plugin: Arc<P>) {
        let mut guard = self.plugins.write().unwrap();
        if guard.insert(TypeId::of::<P>(), plugin as Arc<Plugin>).is_some() {
            panic!("registering a plugin that's already registered");
        }
    }

    pub fn get<P: Plugin>(&self) -> Option<PluginRef<P>> {
        let guard = self.plugins.read().unwrap();
        let arc = guard.get(&TypeId::of::<P>());
        arc.map(|arc| PluginRef {
            inner: arc.clone(),
            _marker: PhantomData
        })
    }
}

#[derive(Clone)]
pub struct PluginRef<P: Plugin> {
    inner: Arc<Plugin>,
    _marker: PhantomData<P>,
}

impl<P: Plugin> Deref for PluginRef<P> {
    type Target = P;

    fn deref(&self) -> &P {
        self.inner.as_any().downcast_ref::<P>().expect("plugin downcast failure")
    }
}

impl<P: Plugin> AsRef<P> for PluginRef<P> {
    fn as_ref(&self) -> &P {
        self.inner.as_any().downcast_ref::<P>().expect("plugin downcast failure")
    }
}

#[derive(Clone)]
pub struct PluginProxyBinding {
    dispatcher: Arc<Dispatcher>,
    plugin_container: Arc<PluginContainer>,
}

impl PluginProxyBinding {
    pub fn new(dispatcher: Arc<Dispatcher>, plugin_container: Arc<PluginContainer>) -> PluginProxyBinding {
        PluginProxyBinding {
            dispatcher: dispatcher,
            plugin_container: plugin_container,
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
            binding.dispatcher.dispatch(event);
        });
    }

    /// Registers an event handler.
    pub fn register_handler<E, F>(&self, priority: Priority, func: F)
        where
            E: Event,
            F: Fn(&E) -> Propagation + 'static {
        self.with_binding(move |binding| {
            // TODO: proper error handling
            binding.dispatcher.register(priority, func);
        });
    }

    /// Tries to get another plugin.
    pub fn plugin<P: Plugin>(&self) -> Option<PluginRef<P>> {
        self.with_binding(|binding| {
            binding.plugin_container.get::<P>()
        })
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
    fn init(dispatcher: &Dispatcher, me: Arc<Plugin>);
}

pub trait PluginAny {
    fn as_any(&self) -> &Any;
}

impl<T: Any + Sized + Plugin> PluginAny for T {
    fn as_any(&self) -> &Any { self }
}
