use plugin::{Plugin, PluginReturn, PluginProxy};
use event::Event;
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
        elem.append_child(Element::builder("body").text(body).build());
        self.proxy.send(elem);
        Ok(())
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
