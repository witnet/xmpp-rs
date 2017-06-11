use plugin::PluginProxy;
use event::{SendElement, ReceiveElement, Propagation, Priority};
use chrono::Local;

pub struct StanzaDebugPlugin {
    proxy: PluginProxy,
}

impl StanzaDebugPlugin {
    pub fn new() -> StanzaDebugPlugin {
        StanzaDebugPlugin {
            proxy: PluginProxy::new(),
        }
    }

    fn handle_send_element(&self, evt: &SendElement) -> Propagation {
        println!("{} [36;1mSEND[0m: {:?}", Local::now(), evt.0);
        Propagation::Continue
    }

    fn handle_receive_element(&self, evt: &ReceiveElement) -> Propagation {
        println!("{} [33;1mRECV[0m: {:?}", Local::now(), evt.0);
        Propagation::Continue
    }
}

impl_plugin!(StanzaDebugPlugin, proxy, [
    (SendElement, Priority::Min) => handle_send_element,
    (ReceiveElement, Priority::Max) => handle_receive_element,
]);
