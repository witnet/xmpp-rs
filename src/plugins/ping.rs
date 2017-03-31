use plugin::{Plugin, PluginReturn, PluginProxy};
use event::Event;
use minidom::Element;
use error::Error;
use jid::Jid;
use ns;

#[derive(Debug)]
pub struct PingEvent {
    pub from: Jid,
    pub to: Jid,
    pub id: String,
}

impl Event for PingEvent {}

pub struct PingPlugin {
    proxy: PluginProxy,
}

impl PingPlugin {
    pub fn new() -> PingPlugin {
        PingPlugin {
            proxy: PluginProxy::new(),
        }
    }

    pub fn send_ping(&self, to: &Jid) -> Result<(), Error> {
        let mut elem = Element::builder("iq")
                               .attr("type", "get")
                               .attr("to", to.to_string())
                               .build();
        elem.append_child(Element::builder("ping").ns(ns::PING).build());
        self.proxy.send(elem);
        Ok(())
    }

    pub fn reply_ping(&self, event: &PingEvent) {
        let reply = Element::builder("iq")
                            .attr("type", "result")
                            .attr("to", event.from.to_string())
                            .attr("id", event.id.to_string())
                            .build();
        self.proxy.send(reply);
    }
}

impl Plugin for PingPlugin {
    fn get_proxy(&mut self) -> &mut PluginProxy {
        &mut self.proxy
    }

    fn handle(&mut self, elem: &Element) -> PluginReturn {
        if elem.is("iq", ns::CLIENT) && elem.attr("type") == Some("get") {
            if elem.has_child("ping", ns::PING) {
                self.proxy.dispatch(PingEvent { // TODO: safety!!!
                    from: elem.attr("from").unwrap().parse().unwrap(),
                    to: elem.attr("to").unwrap().parse().unwrap(),
                    id: elem.attr("id").unwrap().parse().unwrap(),
                });
            }
        }
        PluginReturn::Continue
    }
}
