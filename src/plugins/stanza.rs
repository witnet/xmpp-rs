use std::fmt::Debug;
use std::any::Any;

use plugin::{Plugin, PluginReturn, PluginProxy};
use event::Event;
use minidom::Element;
use jid::Jid;
use ns;

pub trait Stanza: Any + Debug {}

#[derive(Debug)]
pub struct MessageEvent {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: Option<String>,
    pub payloads: Vec<Element>,
}

#[derive(Debug)]
pub struct IqEvent {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: Option<String>,
    pub payloads: Vec<Element>,
}

#[derive(Debug)]
pub struct PresenceEvent {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: Option<String>,
    pub payloads: Vec<Element>,
}

impl Event for MessageEvent {}
impl Event for IqEvent {}
impl Event for PresenceEvent {}

pub struct StanzaPlugin {
    proxy: PluginProxy,
}

impl StanzaPlugin {
    pub fn new() -> StanzaPlugin {
        StanzaPlugin {
            proxy: PluginProxy::new(),
        }
    }
}

impl Plugin for StanzaPlugin {
    fn get_proxy(&mut self) -> &mut PluginProxy {
        &mut self.proxy
    }

    fn handle(&mut self, elem: &Element) -> PluginReturn {
        let from = match elem.attr("from") { Some(from) => Some(from.parse().unwrap()), None => None };
        let to = match elem.attr("to") { Some(to) => Some(to.parse().unwrap()), None => None };
        let id = match elem.attr("id") { Some(id) => Some(id.parse().unwrap()), None => None };
        let type_ = match elem.attr("type") { Some(type_) => Some(type_.parse().unwrap()), None => None };
        let payloads = elem.children().cloned().collect::<Vec<_>>();

        if elem.is("message", ns::CLIENT) {
            self.proxy.dispatch(MessageEvent {
                from: from,
                to: to,
                id: id,
                type_: type_,
                payloads: payloads,
            });
        } else if elem.is("presence", ns::CLIENT) {
            self.proxy.dispatch(PresenceEvent {
                from: from,
                to: to,
                id: id,
                type_: type_,
                payloads: payloads,
            });
        } else if elem.is("iq", ns::CLIENT) {
            self.proxy.dispatch(IqEvent {
                from: from,
                to: to,
                id: id,
                type_: type_,
                payloads: payloads,
            });
        }
        PluginReturn::Continue
    }
}
