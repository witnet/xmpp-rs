use std::collections::BTreeMap;
use std::convert::TryFrom;

use jid::Jid;
use error::Error;
use plugin::PluginProxy;

use event::{Event, Propagation, Priority};

pub use xmpp_parsers::muc::{Muc, MucUser};
pub use xmpp_parsers::muc::user::{Status, Affiliation, Role};
pub use xmpp_parsers::presence::{Presence, Type, Show};

#[derive(Debug)]
pub struct MucPresence {
    pub room: Jid,
    pub nick: Option<String>,
    pub to: Jid,
    pub type_: Type,
    pub x: MucUser,
}

impl Event for MucPresence {}

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
        let x = Muc { password: None };
        let presence = Presence {
            from: None,
            to: Some(room),
            id: None,
            type_: Type::None,
            show: Show::None,
            priority: 0i8,
            statuses: BTreeMap::new(),
            payloads: vec![x.into()],
        };
        self.proxy.send(presence.into());

        Ok(())
    }

    pub fn leave_room(&self, room: Jid) -> Result<(), Error> {
        let x = Muc { password: None };
        let presence = Presence {
            from: None,
            to: Some(room),
            id: None,
            type_: Type::None,
            show: Show::None,
            priority: 0i8,
            statuses: BTreeMap::new(),
            payloads: vec![x.into()],
        };
        self.proxy.send(presence.into());
        Ok(())
    }

    fn handle_presence(&self, presence: &Presence) -> Propagation {
        let from = presence.from.clone().unwrap();
        let room = from.clone().into_bare_jid();
        let nick = from.resource;
        let to = presence.to.clone().unwrap();
        let type_ = presence.type_.clone();

        for payload in presence.clone().payloads {
            if let Ok(x) = MucUser::try_from(payload) {
                self.proxy.dispatch(MucPresence {
                    room: room.clone(),
                    nick: nick.clone(),
                    to: to.clone(),
                    type_: type_.clone(),
                    x
                });
            }
        }

        Propagation::Stop
    }
}

impl_plugin!(MucPlugin, proxy, [
    (Presence, Priority::Default) => handle_presence,
]);
