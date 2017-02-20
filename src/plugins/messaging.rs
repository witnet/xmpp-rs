use plugin::{Plugin, PluginReturn, PluginProxy};
use event::Event;
use minidom::Element;
use jid::Jid;
use ns;

#[derive(Debug)]
pub struct MessageEvent {
    from: Jid,
    to: Jid,
    body: String,
}

impl Event for MessageEvent {}

pub struct MessagingPlugin {
    proxy: PluginProxy,
}

impl MessagingPlugin {
    pub fn new() -> MessagingPlugin {
        MessagingPlugin {
            proxy: PluginProxy::new(),
        }
    }
}

impl Plugin for MessagingPlugin {
    fn get_proxy(&mut self) -> &mut PluginProxy {
        &mut self.proxy
    }

    fn handle(&mut self, elem: &Element) -> PluginReturn {
        if elem.is("message", ns::CLIENT) && elem.attr("type") == Some("chat") {
            if let Some(body) = elem.get_child("body", ns::CLIENT) {
                self.proxy.dispatch(MessageEvent { // TODO: safety!!!
                    from: elem.attr("from").unwrap().parse().unwrap(),
                    to: elem.attr("to").unwrap().parse().unwrap(),
                    body: body.text(),
                });
            }
        }
        PluginReturn::Continue
    }
}
