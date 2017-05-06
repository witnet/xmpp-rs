// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};

use jid::Jid;

use error::Error;

use ns;

use body;
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
    Body(body::Body),
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

#[derive(Debug, Clone)]
pub struct Message {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: MessageType,
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
        let mut payloads = vec!();
        for elem in root.children() {
            let payload = if let Ok(body) = body::parse_body(elem) {
                Some(MessagePayload::Body(body))
            } else if let Ok(stanza_error) = StanzaError::try_from(elem) {
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
        Ok(Message {
            from: from,
            to: to,
            id: id,
            type_: type_,
            payloads: payloads,
        })
    }
}

impl<'a> Into<Element> for &'a MessagePayload {
    fn into(self) -> Element {
        match *self {
            MessagePayload::Body(ref body) => body::serialise(body),
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
            payloads: vec!(),
        };
        let elem2 = (&message).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        Message::try_from(&elem).unwrap();
    }

    #[test]
    fn test_serialise_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let message = Message {
            from: None,
            to: Some(Jid::from_str("coucou@example.org").unwrap()),
            id: None,
            type_: MessageType::Chat,
            payloads: vec!(
                MessagePayloadType::Parsed(MessagePayload::Body("Hello world!".to_owned())),
            ),
        };
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
