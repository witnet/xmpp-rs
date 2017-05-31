use std::collections::BTreeMap;

use jid::Jid;
use error::Error;
use plugin::PluginProxy;

pub use xmpp_parsers::muc::Muc;
pub use xmpp_parsers::presence::{Presence, Type, Show};

pub struct MucPlugin {
    proxy: PluginProxy,
}

impl MucPlugin {
    pub fn new() -> MucPlugin {
        MucPlugin {
            proxy: PluginProxy::new(),
        }
    }

    pub fn join_room(&self, room: Jid) -> Result<(), Error> {
        let presence = Presence {
            from: None,
            to: Some(room),
            id: None,
            type_: Type::None,
            show: Show::None,
            priority: 0i8,
            statuses: BTreeMap::new(),
            payloads: vec![Muc.into()],
        };
        self.proxy.send(presence.into());

        Ok(())
    }

    pub fn leave_room(&self, room: Jid) -> Result<(), Error> {
        let presence = Presence {
            from: None,
            to: Some(room),
            id: None,
            type_: Type::None,
            show: Show::None,
            priority: 0i8,
            statuses: BTreeMap::new(),
            payloads: vec![Muc.into()],
        };
        self.proxy.send(presence.into());
        Ok(())
    }
}

impl_plugin!(MucPlugin, proxy, []);
