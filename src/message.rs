// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::str::FromStr;
use std::collections::BTreeMap;

use minidom::{Element, IntoAttributeValue};

use jid::Jid;

use error::Error;

use ns;

use stanza_error::StanzaError;
use chatstates::ChatState;
use receipts::Receipt;
use delay::Delay;
use attention::Attention;
use message_correct::Replace;
use eme::ExplicitMessageEncryption;

/// Lists every known payload of a `<message/>`.
#[derive(Debug, Clone)]
pub enum MessagePayload {
    StanzaError(StanzaError),
    ChatState(ChatState),
    Receipt(Receipt),
    Delay(Delay),
    Attention(Attention),
    MessageCorrect(Replace),
    ExplicitMessageEncryption(ExplicitMessageEncryption),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Chat,
    Error,
    Groupchat,
    Headline,
    Normal,
}

impl Default for MessageType {
    fn default() -> MessageType {
        MessageType::Normal
    }
}

impl FromStr for MessageType {
    type Err = Error;

    fn from_str(s: &str) -> Result<MessageType, Error> {
        Ok(match s {
            "chat" => MessageType::Chat,
            "error" => MessageType::Error,
            "groupchat" => MessageType::Groupchat,
            "headline" => MessageType::Headline,
            "normal" => MessageType::Normal,

            _ => return Err(Error::ParseError("Invalid 'type' attribute on message element.")),
        })
    }
}

impl IntoAttributeValue for MessageType {
    fn into_attribute_value(self) -> Option<String> {
        Some(match self {
            MessageType::Chat => "chat",
            MessageType::Error => "error",
            MessageType::Groupchat => "groupchat",
            MessageType::Headline => "headline",
            MessageType::Normal => "normal",
        }.to_owned())
    }
}

#[derive(Debug, Clone)]
pub enum MessagePayloadType {
    XML(Element),
    Parsed(MessagePayload),
}

type Lang = String;
type Body = String;
type Subject = String;
type Thread = String;

#[derive(Debug, Clone)]
pub struct Message {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: MessageType,
    pub bodies: BTreeMap<Lang, Body>,
    pub subjects: BTreeMap<Lang, Subject>,
    pub thread: Option<Thread>,
    pub payloads: Vec<MessagePayloadType>,
}

impl<'a> TryFrom<&'a Element> for Message {
    type Error = Error;

    fn try_from(root: &'a Element) -> Result<Message, Error> {
        if !root.is("message", ns::JABBER_CLIENT) {
            return Err(Error::ParseError("This is not a message element."));
        }
        let from = root.attr("from")
            .and_then(|value| value.parse().ok());
        let to = root.attr("to")
            .and_then(|value| value.parse().ok());
        let id = root.attr("id")
            .and_then(|value| value.parse().ok());
        let type_ = match root.attr("type") {
            Some(type_) => type_.parse()?,
            None => Default::default(),
        };
        let mut bodies = BTreeMap::new();
        let mut subjects = BTreeMap::new();
        let mut thread = None;
        let mut payloads = vec!();
        for elem in root.children() {
            if elem.is("body", ns::JABBER_CLIENT) {
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in body element."));
                }
                let lang = elem.attr("xml:lang").unwrap_or("").to_owned();
                if let Some(_) = bodies.insert(lang, elem.text()) {
                    return Err(Error::ParseError("Body element present twice for the same xml:lang."));
                }
            } else if elem.is("subject", ns::JABBER_CLIENT) {
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in subject element."));
                }
                let lang = elem.attr("xml:lang").unwrap_or("").to_owned();
                if let Some(_) = subjects.insert(lang, elem.text()) {
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
                let payload = if let Ok(stanza_error) = StanzaError::try_from(elem) {
                    Some(MessagePayload::StanzaError(stanza_error))
                } else if let Ok(chatstate) = ChatState::try_from(elem) {
                    Some(MessagePayload::ChatState(chatstate))
                } else if let Ok(receipt) = Receipt::try_from(elem) {
                    Some(MessagePayload::Receipt(receipt))
                } else if let Ok(delay) = Delay::try_from(elem) {
                    Some(MessagePayload::Delay(delay))
                } else if let Ok(attention) = Attention::try_from(elem) {
                    Some(MessagePayload::Attention(attention))
                } else if let Ok(replace) = Replace::try_from(elem) {
                    Some(MessagePayload::MessageCorrect(replace))
                } else if let Ok(eme) = ExplicitMessageEncryption::try_from(elem) {
                    Some(MessagePayload::ExplicitMessageEncryption(eme))
                } else {
                    None
                };
                payloads.push(match payload {
                    Some(payload) => MessagePayloadType::Parsed(payload),
                    None => MessagePayloadType::XML(elem.clone()),
                });
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

impl<'a> Into<Element> for &'a MessagePayload {
    fn into(self) -> Element {
        match *self {
            MessagePayload::StanzaError(ref stanza_error) => stanza_error.into(),
            MessagePayload::Attention(ref attention) => attention.into(),
            MessagePayload::ChatState(ref chatstate) => chatstate.into(),
            MessagePayload::Receipt(ref receipt) => receipt.into(),
            MessagePayload::Delay(ref delay) => delay.into(),
            MessagePayload::MessageCorrect(ref replace) => replace.into(),
            MessagePayload::ExplicitMessageEncryption(ref eme) => eme.into(),
        }
    }
}

impl<'a> Into<Element> for &'a Message {
    fn into(self) -> Element {
        let mut stanza = Element::builder("message")
                                 .ns(ns::JABBER_CLIENT)
                                 .attr("from", self.from.clone().and_then(|value| Some(String::from(value))))
                                 .attr("to", self.to.clone().and_then(|value| Some(String::from(value))))
                                 .attr("id", self.id.clone())
                                 .attr("type", self.type_.clone())
                                 .append(self.subjects.iter()
                                                    .map(|(lang, subject)| {
                                                         Element::builder("subject")
                                                                 .ns(ns::JABBER_CLIENT)
                                                                 .attr("xml:lang", match lang.as_ref() {
                                                                      "" => None,
                                                                      lang => Some(lang),
                                                                  })
                                                                 .append(subject.clone())
                                                                 .build() })
                                                    .collect::<Vec<_>>())
                                 .append(self.bodies.iter()
                                                    .map(|(lang, body)| {
                                                         Element::builder("body")
                                                                 .ns(ns::JABBER_CLIENT)
                                                                 .attr("xml:lang", match lang.as_ref() {
                                                                      "" => None,
                                                                      lang => Some(lang),
                                                                  })
                                                                 .append(body.clone())
                                                                 .build() })
                                                    .collect::<Vec<_>>())
                                 .build();
        for child in self.payloads.clone() {
            let elem = match child {
                MessagePayloadType::XML(elem) => elem,
                MessagePayloadType::Parsed(payload) => (&payload).into(),
            };
            stanza.append_child(elem);
        }
        stanza
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        let message = Message::try_from(&elem).unwrap();
        assert_eq!(message.from, None);
        assert_eq!(message.to, None);
        assert_eq!(message.id, None);
        assert_eq!(message.type_, MessageType::Normal);
        assert!(message.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<message xmlns='jabber:client' type='normal'/>".parse().unwrap();
        let message = Message {
            from: None,
            to: None,
            id: None,
            type_: MessageType::Normal,
            bodies: BTreeMap::new(),
            subjects: BTreeMap::new(),
            thread: None,
            payloads: vec!(),
        };
        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let message = Message::try_from(&elem).unwrap();
        assert_eq!(message.bodies[""], "Hello world!");

        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialise_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let mut bodies = BTreeMap::new();
        bodies.insert(String::from(""), String::from("Hello world!"));
        let message = Message {
            from: None,
            to: Some(Jid::from_str("coucou@example.org").unwrap()),
            id: None,
            type_: MessageType::Chat,
            bodies: bodies,
            subjects: BTreeMap::new(),
            thread: None,
            payloads: vec!(),
        };
        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_subject() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        let message = Message::try_from(&elem).unwrap();
        assert_eq!(message.subjects[""], "Hello world!");

        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_attention() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        let message = Message::try_from(&elem).unwrap();
        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }
}
