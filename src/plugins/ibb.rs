use std::collections::{HashMap, BTreeMap};
use std::collections::hash_map::Entry;
use try_from::TryFrom;
use std::sync::{Mutex, Arc};

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use jid::Jid;

use plugins::stanza::Iq;
use plugins::disco::DiscoPlugin;
use xmpp_parsers::iq::{IqType, IqSetPayload};
use xmpp_parsers::ibb::{IBB, Stanza};
use xmpp_parsers::stanza_error::{StanzaError, ErrorType, DefinedCondition};
use xmpp_parsers::ns;

#[derive(Debug, Clone)]
pub struct Session {
    stanza: Stanza,
    block_size: u16,
    cur_seq: u16,
}

#[derive(Debug)]
pub struct IbbOpen {
    pub session: Session,
}

#[derive(Debug)]
pub struct IbbData {
    pub session: Session,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct IbbClose {
    pub session: Session,
}

impl Event for IbbOpen {}
impl Event for IbbData {}
impl Event for IbbClose {}

fn generate_error(type_: ErrorType, defined_condition: DefinedCondition, text: &str) -> StanzaError {
    StanzaError {
        type_: type_,
        defined_condition: defined_condition,
        texts: {
            let mut texts = BTreeMap::new();
            texts.insert(String::new(), String::from(text));
            texts
        },
        by: None,
        other: None,
    }
}

pub struct IbbPlugin {
    proxy: PluginProxy,
    sessions: Arc<Mutex<HashMap<(Jid, String), Session>>>,
}

impl IbbPlugin {
    pub fn new() -> IbbPlugin {
        IbbPlugin {
            proxy: PluginProxy::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
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

    fn handle_ibb(&self, from: Jid, ibb: IBB) -> Result<(), StanzaError> {
        let mut sessions = self.sessions.lock().unwrap();
        match ibb {
            IBB::Open { block_size, sid, stanza } => {
                match sessions.entry((from.clone(), sid.clone())) {
                    Entry::Vacant(_) => Ok(()),
                    Entry::Occupied(_) => Err(generate_error(
                        ErrorType::Cancel,
                        DefinedCondition::NotAcceptable,
                        "This session is already open."
                    )),
                }?;
                let session = Session {
                    stanza,
                    block_size,
                    cur_seq: 65535u16,
                };
                sessions.insert((from, sid), session.clone());
                self.proxy.dispatch(IbbOpen {
                    session: session,
                });
            },
            IBB::Data { seq, sid, data } => {
                let entry = match sessions.entry((from, sid)) {
                    Entry::Occupied(entry) => Ok(entry),
                    Entry::Vacant(_) => Err(generate_error(
                        ErrorType::Cancel,
                        DefinedCondition::ItemNotFound,
                        "This session doesn’t exist."
                    )),
                }?;
                let mut session = entry.into_mut();
                if session.stanza != Stanza::Iq {
                    return Err(generate_error(
                        ErrorType::Cancel,
                        DefinedCondition::NotAcceptable,
                        "Wrong stanza type."
                    ))
                }
                let cur_seq = session.cur_seq.wrapping_add(1);
                if seq != cur_seq {
                    return Err(generate_error(
                        ErrorType::Cancel,
                        DefinedCondition::NotAcceptable,
                        "Wrong seq number."
                    ))
                }
                session.cur_seq = cur_seq;
                self.proxy.dispatch(IbbData {
                    session: session.clone(),
                    data,
                });
            },
            IBB::Close { sid } => {
                let entry = match sessions.entry((from, sid)) {
                    Entry::Occupied(entry) => Ok(entry),
                    Entry::Vacant(_) => Err(generate_error(
                        ErrorType::Cancel,
                        DefinedCondition::ItemNotFound,
                        "This session doesn’t exist."
                    )),
                }?;
                let session = entry.remove();
                self.proxy.dispatch(IbbClose {
                    session,
                });
            },
        }
        Ok(())
    }

    fn handle_iq(&self, iq: &Iq) -> Propagation {
        let iq = iq.clone();
        if let IqType::Set(payload) = iq.payload {
            let from = iq.from.unwrap();
            let id = iq.id.unwrap();
            // TODO: use an intermediate plugin to parse this payload.
            let payload = match IqSetPayload::try_from(payload) {
                Ok(IqSetPayload::IBB(ibb)) => {
                    match self.handle_ibb(from.clone(), ibb) {
                        Ok(_) => IqType::Result(None),
                        Err(error) => IqType::Error(error),
                    }
                },
                Err(err) => IqType::Error(generate_error(
                    ErrorType::Cancel,
                    DefinedCondition::NotAcceptable,
                    format!("{:?}", err).as_ref()
                )),
                Ok(_) => return Propagation::Continue,
            };
            self.proxy.send(Iq {
                from: None,
                to: Some(from),
                id: Some(id),
                payload: payload,
            }.into());
            Propagation::Stop
        } else {
            Propagation::Continue
        }
    }
}

impl_plugin!(IbbPlugin, proxy, [
    (Iq, Priority::Default) => handle_iq,
]);
