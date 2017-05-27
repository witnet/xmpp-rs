use plugin::PluginProxy;
use event::{Event, ReceiveElement, Priority, Propagation};
use minidom::Element;
use error::Error;
use jid::Jid;
use ns;

#[derive(Debug)]
pub struct MessageEvent {
    pub from: Jid,
    pub to: Jid,
    pub body: String,
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

    pub fn send_message(&self, to: &Jid, body: &str) -> Result<(), Error> {
        let mut elem = Element::builder("message")
                               .attr("type", "chat")
                               .attr("to", to.to_string())
                               .build();
        elem.append_child(Element::builder("body").append(body).build());
        self.proxy.send(elem);
        Ok(())
    }

    fn handle_receive_element(&self, evt: &ReceiveElement) -> Propagation {
        let elem = &evt.0;
        if elem.is("message", ns::CLIENT) && elem.attr("type") == Some("chat") {
            if let Some(body) = elem.get_child("body", ns::CLIENT) {
                self.proxy.dispatch(MessageEvent { // TODO: safety!!!
                    from: elem.attr("from").unwrap().parse().unwrap(),
                    to: elem.attr("to").unwrap().parse().unwrap(),
                    body: body.text(),
                });
            }
        }
        Propagation::Continue
    }
}

impl_plugin!(MessagingPlugin, proxy, [
    (ReceiveElement, Priority::Default) => handle_receive_element,
]);
