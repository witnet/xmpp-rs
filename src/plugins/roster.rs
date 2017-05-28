use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use jid::Jid;

use plugins::stanza::Iq;
use plugins::disco::DiscoPlugin;
use xmpp_parsers::iq::{IqType, IqPayload};
use xmpp_parsers::roster::{Roster, Item, Subscription};
use xmpp_parsers::ns;

#[derive(Debug)]
pub struct RosterReceived {
    pub ver: Option<String>,
    pub jids: HashMap<Jid, Item>,
}

#[derive(Debug)]
pub enum RosterPush {
    Added(Item),
    Modified(Item),
    Removed(Item),
}

impl Event for RosterReceived {}
impl Event for RosterPush {}

pub struct RosterPlugin {
    proxy: PluginProxy,
    current_version: Mutex<Option<String>>,
    // TODO: allow for a different backing store.
    jids: Mutex<HashMap<Jid, Item>>,
}

impl RosterPlugin {
    pub fn new(ver: Option<String>) -> RosterPlugin {
        RosterPlugin {
            proxy: PluginProxy::new(),
            current_version: Mutex::new(ver),
            jids: Mutex::new(HashMap::new()),
        }
    }

    // TODO: make that called automatically after plugins are created.
    pub fn init(&self) {
        if let Some(disco) = self.proxy.plugin::<DiscoPlugin>() {
            disco.add_feature(ns::IBB);
        } else {
            panic!("Please handle dependencies in the correct order.");
        }
    }

    // TODO: make that called automatically before removal.
    pub fn deinit(&self) {
        if let Some(disco) = self.proxy.plugin::<DiscoPlugin>() {
            disco.remove_feature(ns::IBB);
        } else {
            panic!("Please handle dependencies in the correct order.");
        }
    }

    pub fn send_roster_get(&self, ver: Option<String>) {
        let iq = Iq {
            from: None,
            to: None,
            id: Some(self.proxy.gen_id()),
            payload: IqType::Get(Roster {
                ver,
                items: vec!(),
            }.into()),
        };
        self.proxy.send(iq.into());
    }

    // TODO: use a better error type.
    pub fn send_roster_set(&self, to: Option<Jid>, item: Item) -> Result<(), String> {
        if item.subscription.is_some() && item.subscription != Some(Subscription::Remove) {
            return Err(String::from("Subscription must be either nothing or Remove."));
        }
        let iq = Iq {
            from: None,
            to,
            id: Some(self.proxy.gen_id()),
            payload: IqType::Set(Roster {
                ver: None,
                items: vec!(item),
            }.into()),
        };
        self.proxy.send(iq.into());
        Ok(())
    }

    fn handle_roster_reply(&self, roster: Roster) {
        // TODO: handle the same-ver case!
        let mut current_version = self.current_version.lock().unwrap();
        *current_version = roster.ver;
        let mut jids = self.jids.lock().unwrap();
        jids.clear();
        for item in roster.items {
            jids.insert(item.jid.clone(), item);
        }
        self.proxy.dispatch(RosterReceived {
            ver: current_version.clone(),
            jids: jids.clone(),
        });
    }

    fn handle_roster_push(&self, roster: Roster) -> Result<(), String> {
        let item = roster.items.get(0);
        if item.is_none() || roster.items.len() != 1 {
            return Err(String::from("Server sent an invalid roster push!"));
        }
        let item = item.unwrap().clone();
        let mut jids = self.jids.lock().unwrap();
        let previous = jids.insert(item.jid.clone(), item.clone());
        if previous.is_none() {
            assert!(item.subscription != Some(Subscription::Remove));
            self.proxy.dispatch(RosterPush::Added(item));
        } else {
            if item.subscription == Some(Subscription::Remove) {
                self.proxy.dispatch(RosterPush::Removed(item));
            } else {
                self.proxy.dispatch(RosterPush::Modified(item));
            }
        }
        Ok(())
    }

    fn handle_iq(&self, iq: &Iq) -> Propagation {
        let jid = self.proxy.get_own_jid();
        let jid = Jid::bare(jid.node.unwrap(), jid.domain);
        if iq.from.is_some() && iq.from != Some(jid) {
            // Not from our roster.
            return Propagation::Continue;
        }
        let iq = iq.clone();
        let id = iq.id.unwrap();
        match iq.payload {
            IqType::Result(Some(payload)) => {
                match IqPayload::try_from(payload) {
                    Ok(IqPayload::Roster(roster)) => {
                        self.handle_roster_reply(roster);
                        Propagation::Stop
                    },
                    Ok(_)
                  | Err(_) => Propagation::Continue,
                }
            },
            IqType::Set(payload) => {
                match IqPayload::try_from(payload) {
                    Ok(IqPayload::Roster(roster)) => {
                        let payload = match self.handle_roster_push(roster) {
                            Ok(_) => IqType::Result(None),
                            Err(string) => {
                                // The specification says that the server should ignore an error.
                                println!("{}", string);
                                IqType::Result(None)
                            },
                        };
                        self.proxy.send(Iq {
                            from: None,
                            to: None,
                            id: Some(id),
                            payload: payload,
                        }.into());
                        Propagation::Stop
                    },
                    Ok(_)
                  | Err(_) => return Propagation::Continue,
                }
            },
            IqType::Result(None)
          | IqType::Get(_)
          | IqType::Error(_) => {
                Propagation::Continue
            },
        }
    }
}

impl_plugin!(RosterPlugin, proxy, [
    (Iq, Priority::Default) => handle_iq,
]);
