use event::{Event, AbstractEvent};

use std::any::Any;

use std::sync::mpsc::Sender;

use std::mem;

use minidom::Element;

#[derive(Clone)]
pub struct PluginProxyBinding {
    sender: Sender<Element>,
    dispatcher: Sender<AbstractEvent>,
}

impl PluginProxyBinding {
    pub fn new(sender: Sender<Element>, dispatcher: Sender<AbstractEvent>) -> PluginProxyBinding {
        PluginProxyBinding {
            sender: sender,
            dispatcher: dispatcher,
        }
    }
}

pub enum PluginProxy {
    Unbound,
    BoundTo(PluginProxyBinding),
}

impl PluginProxy {
    pub fn new() -> PluginProxy {
        PluginProxy::Unbound
    }

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

    pub fn dispatch<E: Event>(&self, event: E) {
        self.with_binding(move |binding| {
            binding.dispatcher.send(AbstractEvent::new(event))
                              .unwrap(); // TODO: may want to return the error
        });
    }

    pub fn send(&self, elem: Element) {
        self.with_binding(move |binding| {
            binding.sender.send(elem).unwrap(); // TODO: as above, may want to return the error
        });
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PluginReturn {
    Continue,
    Unload,
}

pub trait Plugin: Any + PluginAny {
    fn get_proxy(&mut self) -> &mut PluginProxy;
    fn handle(&mut self, _elem: &Element) -> PluginReturn { PluginReturn::Continue }

    fn bind(&mut self, inner: PluginProxyBinding) {
        self.get_proxy().bind(inner);
    }
}

pub trait PluginAny {
    fn as_any(&self) -> &Any;
}

impl<T: Any + Sized> PluginAny for T {
    fn as_any(&self) -> &Any { self }
}
