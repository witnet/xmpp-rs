use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Mutex, Arc};

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use jid::Jid;
use base64;

use plugins::stanza::{Presence, Iq};
use plugins::disco::DiscoInfoResult;
use xmpp_parsers::presence::Type as PresenceType;
use xmpp_parsers::iq::IqType;
use xmpp_parsers::disco::Disco;
use xmpp_parsers::caps::Caps;

#[derive(Debug)]
pub struct DiscoInfoRequest {
    pub from: Jid,
    pub id: String,
    pub node: Option<String>,
}

impl Event for DiscoInfoRequest {}

pub struct CapsPlugin {
    proxy: PluginProxy,
    pending: Arc<Mutex<HashMap<Jid, (String, String)>>>,
    cache: Arc<Mutex<HashMap<(Jid, String), Disco>>>,
}

impl CapsPlugin {
    pub fn new() -> CapsPlugin {
        CapsPlugin {
            proxy: PluginProxy::new(),
            pending: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn handle_presence(&self, presence: &Presence) -> Propagation {
        let presence = presence.clone();
        match presence.type_ {
            PresenceType::None => for payload in presence.payloads {
                let caps = match Caps::try_from(payload) {
                    Ok(caps) => caps,
                    Err(_) => continue,
                };
                let recipient = presence.from.unwrap();
                let node = format!("{}#{}", caps.node, base64::encode(&caps.hash.hash));
                {
                    let cache = self.cache.lock().unwrap();
                    if cache.contains_key(&(recipient.clone(), node.clone())) {
                        break;
                    }
                }
                let id = self.proxy.gen_id();
                {
                    let mut pending = self.pending.lock().unwrap();
                    pending.insert(recipient.clone(), (id.clone(), node.clone()));
                }
                let disco = Disco {
                    node: Some(node),
                    identities: vec!(),
                    features: vec!(),
                    extensions: vec!(),
                };
                self.proxy.send(Iq {
                    to: Some(recipient),
                    from: None,
                    id: Some(id),
                    payload: IqType::Get(disco.into()),
                }.into());
                break;
            },
            PresenceType::Unavailable
          | PresenceType::Error => {
                let recipient = presence.from.unwrap();
                let mut pending = self.pending.lock().unwrap();
                let previous = pending.remove(&recipient);
                if previous.is_none() {
                    // This wasn’t one of our requests.
                    return Propagation::Continue;
                }
                // TODO: maybe add a negative cache?
            },
            _ => (),
        }
        Propagation::Continue
    }

    fn handle_result(&self, result: &DiscoInfoResult) -> Propagation {
        let from = result.from.clone();
        let mut pending = self.pending.lock().unwrap();
        let previous = pending.remove(&from.clone());
        if let Some((id, node)) = previous {
            if id != result.id {
                return Propagation::Continue;
            }
            if Some(node.clone()) != result.disco.node {
                // TODO: make that a debug log.
                println!("Wrong node in result!");
                return Propagation::Continue;
            }
            {
                let mut cache = self.cache.lock().unwrap();
                cache.insert((from, node), result.disco.clone());
            }
        } else {
            // TODO: make that a debug log.
            println!("No such request from us.");
            return Propagation::Continue;
        }
        Propagation::Stop
    }

    // This is only for errors.
    // TODO: also do the same thing for timeouts.
    fn handle_iq(&self, iq: &Iq) -> Propagation {
        let iq = iq.clone();
        if let IqType::Error(_) = iq.payload {
            let from = iq.from.unwrap();
            let mut pending = self.pending.lock().unwrap();
            let previous = pending.remove(&from.clone());
            if previous.is_none() {
                // This wasn’t one of our requests.
                return Propagation::Continue;
            }
            // TODO: maybe add a negative cache?
            return Propagation::Stop;
        }
        Propagation::Continue
    }
}

impl_plugin!(CapsPlugin, proxy, [
    (Presence, Priority::Default) => handle_presence,
    (Iq, Priority::Default) => handle_iq,
    (DiscoInfoResult, Priority::Default) => handle_result,
]);
