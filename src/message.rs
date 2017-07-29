// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;
use std::collections::BTreeMap;

use minidom::{Element, IntoAttributeValue};

use jid::Jid;

use error::Error;

use ns;

use stanza_error::StanzaError;
use chatstates::ChatState;
use receipts::{Request as ReceiptRequest, Received as ReceiptReceived};
use delay::Delay;
use attention::Attention;
use message_correct::Replace;
use eme::ExplicitMessageEncryption;
use stanza_id::{StanzaId, OriginId};
use mam::Result_ as MamResult;

/// Lists every known payload of a `<message/>`.
#[derive(Debug, Clone)]
pub enum MessagePayload {
    StanzaError(StanzaError),
    ChatState(ChatState),
    ReceiptRequest(ReceiptRequest),
    ReceiptReceived(ReceiptReceived),
    Delay(Delay),
    Attention(Attention),
    MessageCorrect(Replace),
    ExplicitMessageEncryption(ExplicitMessageEncryption),
    StanzaId(StanzaId),
    OriginId(OriginId),
    MamResult(MamResult),

    Unknown(Element),
}

impl TryFrom<Element> for MessagePayload {
    type Err = Error;

    fn try_from(elem: Element) -> Result<MessagePayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            ("error", ns::JABBER_CLIENT) => MessagePayload::StanzaError(StanzaError::try_from(elem)?),

            // XEP-0085
            ("active", ns::CHATSTATES)
          | ("inactive", ns::CHATSTATES)
          | ("composing", ns::CHATSTATES)
          | ("paused", ns::CHATSTATES)
          | ("gone", ns::CHATSTATES) => MessagePayload::ChatState(ChatState::try_from(elem)?),

            // XEP-0184
            ("request", ns::RECEIPTS) => MessagePayload::ReceiptRequest(ReceiptRequest::try_from(elem)?),
            ("received", ns::RECEIPTS) => MessagePayload::ReceiptReceived(ReceiptReceived::try_from(elem)?),

            // XEP-0203
            ("delay", ns::DELAY) => MessagePayload::Delay(Delay::try_from(elem)?),

            // XEP-0224
            ("attention", ns::ATTENTION) => MessagePayload::Attention(Attention::try_from(elem)?),

            // XEP-0308
            ("replace", ns::MESSAGE_CORRECT) => MessagePayload::MessageCorrect(Replace::try_from(elem)?),

            // XEP-0313
            ("result", ns::MAM) => MessagePayload::MamResult(MamResult::try_from(elem)?),

            // XEP-0359
            ("stanza-id", ns::SID) => MessagePayload::StanzaId(StanzaId::try_from(elem)?),
            ("origin-id", ns::SID) => MessagePayload::OriginId(OriginId::try_from(elem)?),

            // XEP-0380
            ("encryption", ns::EME) => MessagePayload::ExplicitMessageEncryption(ExplicitMessageEncryption::try_from(elem)?),

            _ => MessagePayload::Unknown(elem),
        })
    }
}

impl From<MessagePayload> for Element {
    fn from(payload: MessagePayload) -> Element {
        match payload {
            MessagePayload::StanzaError(stanza_error) => stanza_error.into(),
            MessagePayload::Attention(attention) => attention.into(),
            MessagePayload::ChatState(chatstate) => chatstate.into(),
            MessagePayload::ReceiptRequest(request) => request.into(),
            MessagePayload::ReceiptReceived(received) => received.into(),
            MessagePayload::Delay(delay) => delay.into(),
            MessagePayload::MessageCorrect(replace) => replace.into(),
            MessagePayload::ExplicitMessageEncryption(eme) => eme.into(),
            MessagePayload::StanzaId(stanza_id) => stanza_id.into(),
            MessagePayload::OriginId(origin_id) => origin_id.into(),
            MessagePayload::MamResult(result) => result.into(),

            MessagePayload::Unknown(elem) => elem,
        }
    }
}

generate_attribute!(MessageType, "type", {
    Chat => "chat",
    Error => "error",
    Groupchat => "groupchat",
    Headline => "headline",
    Normal => "normal",
}, Default = Normal);

type Lang = String;
type Body = String;
type Subject = String;
type Thread = String;

/// The main structure representing the `<message/>` stanza.
#[derive(Debug, Clone)]
pub struct Message {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: MessageType,
    pub bodies: BTreeMap<Lang, Body>,
    pub subjects: BTreeMap<Lang, Subject>,
    pub thread: Option<Thread>,
    pub payloads: Vec<Element>,
}

impl Message {
    pub fn new(to: Option<Jid>) -> Message {
        Message {
            from: None,
            to: to,
            id: None,
            type_: MessageType::Chat,
            bodies: BTreeMap::new(),
            subjects: BTreeMap::new(),
            thread: None,
            payloads: vec!(),
        }
    }
}

impl TryFrom<Element> for Message {
    type Err = Error;

    fn try_from(root: Element) -> Result<Message, Error> {
        if !root.is("message", ns::JABBER_CLIENT) {
            return Err(Error::ParseError("This is not a message element."));
        }
        let from = get_attr!(root, "from", optional);
        let to = get_attr!(root, "to", optional);
        let id = get_attr!(root, "id", optional);
        let type_ = get_attr!(root, "type", default);
        let mut bodies = BTreeMap::new();
        let mut subjects = BTreeMap::new();
        let mut thread = None;
        let mut payloads = vec!();
        for elem in root.children() {
            if elem.is("body", ns::JABBER_CLIENT) {
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in body element."));
                }
                let lang = get_attr!(root, "xml:lang", default);
                if bodies.insert(lang, elem.text()).is_some() {
                    return Err(Error::ParseError("Body element present twice for the same xml:lang."));
                }
            } else if elem.is("subject", ns::JABBER_CLIENT) {
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in subject element."));
                }
                let lang = get_attr!(root, "xml:lang", default);
                if subjects.insert(lang, elem.text()).is_some() {
                    return Err(Error::ParseError("Subject element present twice for the same xml:lang."));
                }
            } else if elem.is("thread", ns::JABBER_CLIENT) {
                if thread.is_some() {
                    return Err(Error::ParseError("Thread element present twice."));
                }
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in thread element."));
                }
                thread = Some(elem.text());
            } else {
                payloads.push(elem.clone())
            }
        }
        Ok(Message {
            from: from,
            to: to,
            id: id,
            type_: type_,
            bodies: bodies,
            subjects: subjects,
            thread: thread,
            payloads: payloads,
        })
    }
}

impl From<Message> for Element {
    fn from(message: Message) -> Element {
        Element::builder("message")
                .ns(ns::JABBER_CLIENT)
                .attr("from", message.from.map(String::from))
                .attr("to", message.to.map(String::from))
                .attr("id", message.id)
                .attr("type", message.type_)
                .append(message.subjects.into_iter()
                                        .map(|(lang, subject)| {
                                             Element::builder("subject")
                                                     .ns(ns::JABBER_CLIENT)
                                                     .attr("xml:lang", match lang.as_ref() {
                                                          "" => None,
                                                          lang => Some(lang),
                                                      })
                                                     .append(subject)
                                                     .build() })
                                        .collect::<Vec<_>>())
                .append(message.bodies.into_iter()
                                      .map(|(lang, body)| {
                                           Element::builder("body")
                                                   .ns(ns::JABBER_CLIENT)
                                                   .attr("xml:lang", match lang.as_ref() {
                                                        "" => None,
                                                        lang => Some(lang),
                                                    })
                                                   .append(body)
                                                   .build() })
                                      .collect::<Vec<_>>())
                .append(message.payloads)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.from, None);
        assert_eq!(message.to, None);
        assert_eq!(message.id, None);
        assert_eq!(message.type_, MessageType::Normal);
        assert!(message.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        let mut message = Message::new(None);
        message.type_ = MessageType::Normal;
        let elem2 = message.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.bodies[""], "Hello world!");

        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_serialise_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let mut message = Message::new(Some(Jid::from_str("coucou@example.org").unwrap()));
        message.bodies.insert(String::from(""), String::from("Hello world!"));
        let elem2 = message.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_subject() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.subjects[""], "Hello world!");

        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_attention() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }
}
