use try_from::TryFrom;

use plugin::PluginProxy;
use event::{Event, ReceiveElement, Propagation, Priority};
use ns;

pub use xmpp_parsers::message::Message;
pub use xmpp_parsers::presence::Presence;
pub use xmpp_parsers::iq::Iq;

impl Event for Message {}
impl Event for Presence {}
impl Event for Iq {}

pub struct StanzaPlugin {
    proxy: PluginProxy,
}

impl StanzaPlugin {
    pub fn new() -> StanzaPlugin {
        StanzaPlugin {
            proxy: PluginProxy::new(),
        }
    }

    fn handle_receive_element(&self, evt: &ReceiveElement) -> Propagation {
        let elem = &evt.0;

        // TODO: make the handle take an Element instead of a reference.
        let elem = elem.clone();

        if elem.is("message", ns::CLIENT) {
            let message = Message::try_from(elem).unwrap();
            self.proxy.dispatch(message);
        } else if elem.is("presence", ns::CLIENT) {
            let presence = Presence::try_from(elem).unwrap();
            self.proxy.dispatch(presence);
        } else if elem.is("iq", ns::CLIENT) {
            let iq = Iq::try_from(elem).unwrap();
            self.proxy.dispatch(iq);
        } else {
            // TODO: handle nonzas too.
            return Propagation::Continue;
        }

        Propagation::Stop
    }
}

impl_plugin!(StanzaPlugin, proxy, [
    (ReceiveElement, Priority::Default) => handle_receive_element,
]);
