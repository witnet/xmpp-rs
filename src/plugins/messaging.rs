use std::convert::TryFrom;
use std::collections::BTreeMap;

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use error::Error;
use jid::Jid;

use plugins::stanza::Message;
use xmpp_parsers::message::{MessagePayload, MessageType};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::receipts::Receipt;
use xmpp_parsers::stanza_id::StanzaId;

// TODO: use the id (maybe even stanza-id) to identify every message.
#[derive(Debug)]
pub struct MessageEvent {
    pub from: Jid,
    pub body: String,
    pub subject: Option<String>,
    pub thread: Option<String>,
}

#[derive(Debug)]
pub struct ChatStateEvent {
    pub from: Jid,
    pub chat_state: ChatState,
}

#[derive(Debug)]
pub struct ReceiptRequestEvent {
    pub from: Jid,
}

#[derive(Debug)]
pub struct ReceiptReceivedEvent {
    pub from: Jid,
    pub id: String,
}

#[derive(Debug)]
pub struct StanzaIdEvent {
    pub from: Jid,
    pub stanza_id: StanzaId,
    pub message: Message,
}

impl Event for MessageEvent {}
impl Event for ChatStateEvent {}
impl Event for ReceiptRequestEvent {}
impl Event for ReceiptReceivedEvent {}
impl Event for StanzaIdEvent {}

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
        let message = Message {
            from: None,
            to: Some(to.clone()),
            type_: MessageType::Chat,
            id: Some(self.proxy.gen_id()),
            bodies: {
                let mut bodies = BTreeMap::new();
                bodies.insert(String::new(), String::from(body));
                bodies
            },
            subjects: BTreeMap::new(),
            thread: None,
            payloads: vec!(),
        };
        self.proxy.send(message.into());
        Ok(())
    }

    fn handle_message(&self, message: &Message) -> Propagation {
        let from = message.from.clone().unwrap();
        for payload in message.payloads.clone() {
            let payload = match MessagePayload::try_from(payload) {
                Ok(payload) => payload,
                Err(err) => {
                    println!("MessagePayload: {:?}", err);
                    continue;
                }
            };
            match payload {
                // XEP-0085
                MessagePayload::ChatState(chat_state) => self.proxy.dispatch(ChatStateEvent {
                    from: from.clone(),
                    chat_state: chat_state,
                }),
                // XEP-0184
                MessagePayload::Receipt(Receipt::Request) => self.proxy.dispatch(ReceiptRequestEvent {
                    from: from.clone(),
                }),
                // XEP-0184
                MessagePayload::Receipt(Receipt::Received(id)) => self.proxy.dispatch(ReceiptReceivedEvent {
                    from: from.clone(),
                    id: id.unwrap(),
                }),
                // XEP-0359
                MessagePayload::StanzaId(stanza_id) => self.proxy.dispatch(StanzaIdEvent {
                    from: from.clone(),
                    stanza_id: stanza_id,
                    message: message.clone(),
                }),
                payload => println!("Unhandled payload: {:?}", payload),
            }
        }
        if message.bodies.contains_key("") {
            self.proxy.dispatch(MessageEvent {
                from: from,
                body: message.bodies[""].clone(),
                subject: if message.subjects.contains_key("") { Some(message.subjects[""].clone()) } else { None },
                thread: message.thread.clone(),
            });
        }
        Propagation::Stop
    }
}

impl_plugin!(MessagingPlugin, proxy, [
    (Message, Priority::Default) => handle_message,
]);
