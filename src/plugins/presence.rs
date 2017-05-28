use std::collections::BTreeMap;

use error::Error;
use plugin::PluginProxy;

pub use xmpp_parsers::presence::{Presence, Type, Show};

pub struct PresencePlugin {
    proxy: PluginProxy,
}

impl PresencePlugin {
    pub fn new() -> PresencePlugin {
        PresencePlugin {
            proxy: PluginProxy::new(),
        }
    }

    pub fn set_presence(&self, type_: Type, show: Show, status: Option<String>) -> Result<(), Error> {
        let presence = Presence {
            from: None,
            to: None,
            id: Some(self.proxy.gen_id()),
            type_: type_,
            show: show,
            priority: 0i8,
            statuses: {
                let mut statuses = BTreeMap::new();
                if let Some(status) = status {
                    statuses.insert(String::new(), status);
                }
                statuses
            },
            payloads: vec!(),
        };
        self.proxy.send(presence.into());
        Ok(())
    }
}

impl_plugin!(PresencePlugin, proxy, []);
