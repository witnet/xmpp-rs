use plugin::PluginProxy;
use event::{Event, EventHandler, Priority, Propagation, ReceiveElement};
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

impl_plugin!(PingPlugin, proxy, [
    ReceiveElement => Priority::Default,
]);

impl EventHandler<ReceiveElement> for PingPlugin {
    fn handle(&self, evt: &ReceiveElement) -> Propagation {
        let elem = &evt.0;
        if elem.is("iq", ns::CLIENT) && elem.attr("type") == Some("get") {
            if elem.has_child("ping", ns::PING) {
                self.proxy.dispatch(PingEvent { // TODO: safety!!!
                    from: elem.attr("from").unwrap().parse().unwrap(),
                    to: elem.attr("to").unwrap().parse().unwrap(),
                    id: elem.attr("id").unwrap().parse().unwrap(),
                });
            }
        }
        Propagation::Continue
    }
}
