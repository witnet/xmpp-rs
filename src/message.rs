// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoElements, IntoAttributeValue};
use minidom::convert::ElementEmitter;

use jid::Jid;

use error::Error;

use ns;

use body;
use stanza_error;
use chatstates;
use receipts;
use delay;
use attention::Attention;
use message_correct;
use eme;

/// Lists every known payload of a `<message/>`.
#[derive(Debug, Clone)]
pub enum MessagePayload {
    Body(body::Body),
    StanzaError(stanza_error::StanzaError),
    ChatState(chatstates::ChatState),
    Receipt(receipts::Receipt),
    Delay(delay::Delay),
    Attention(Attention),
    MessageCorrect(message_correct::Replace),
    ExplicitMessageEncryption(eme::ExplicitMessageEncryption),
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

pub fn parse_message(root: &Element) -> Result<Message, Error> {
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
        } else if let Ok(stanza_error) = stanza_error::parse_stanza_error(elem) {
            Some(MessagePayload::StanzaError(stanza_error))
        } else if let Ok(chatstate) = chatstates::parse_chatstate(elem) {
            Some(MessagePayload::ChatState(chatstate))
        } else if let Ok(receipt) = receipts::parse_receipt(elem) {
            Some(MessagePayload::Receipt(receipt))
        } else if let Ok(delay) = delay::parse_delay(elem) {
            Some(MessagePayload::Delay(delay))
        } else if let Ok(attention) = Attention::try_from(elem) {
            Some(MessagePayload::Attention(attention))
        } else if let Ok(replace) = message_correct::parse_replace(elem) {
            Some(MessagePayload::MessageCorrect(replace))
        } else if let Ok(eme) = eme::parse_explicit_message_encryption(elem) {
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

pub fn serialise_payload(payload: &MessagePayload) -> Element {
    match *payload {
        MessagePayload::Body(ref body) => body::serialise(body),
        MessagePayload::StanzaError(ref stanza_error) => stanza_error::serialise(stanza_error),
        MessagePayload::Attention(ref attention) => attention.into(),
        MessagePayload::ChatState(ref chatstate) => chatstates::serialise(chatstate),
        MessagePayload::Receipt(ref receipt) => receipts::serialise(receipt),
        MessagePayload::Delay(ref delay) => delay::serialise(delay),
        MessagePayload::MessageCorrect(ref replace) => message_correct::serialise(replace),
        MessagePayload::ExplicitMessageEncryption(ref eme) => eme::serialise(eme),
    }
}

pub fn serialise(message: &Message) -> Element {
    let mut stanza = Element::builder("message")
                             .ns(ns::JABBER_CLIENT)
                             .attr("from", message.from.clone().and_then(|value| Some(String::from(value))))
                             .attr("to", message.to.clone().and_then(|value| Some(String::from(value))))
                             .attr("id", message.id.clone())
                             .attr("type", message.type_.clone())
                             .build();
    for child in message.payloads.clone() {
        let elem = match child {
            MessagePayloadType::XML(elem) => elem,
            MessagePayloadType::Parsed(payload) => serialise_payload(&payload),
        };
        stanza.append_child(elem);
    }
    stanza
}

impl IntoElements for Message {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let elem = serialise(&self);
        emitter.append_child(elem);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use minidom::Element;
    use jid::Jid;
    use message;

    #[test]
    fn test_simple() {
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        let message = message::parse_message(&elem).unwrap();
        assert_eq!(message.from, None);
        assert_eq!(message.to, None);
        assert_eq!(message.id, None);
        assert_eq!(message.type_, message::MessageType::Normal);
        assert!(message.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<message xmlns='jabber:client' type='normal'/>".parse().unwrap();
        let message = message::Message {
            from: None,
            to: None,
            id: None,
            type_: message::MessageType::Normal,
            payloads: vec!(),
        };
        let elem2 = message::serialise(&message);
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        message::parse_message(&elem).unwrap();
    }

    #[test]
    fn test_serialise_body() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let message = message::Message {
            from: None,
            to: Some(Jid::from_str("coucou@example.org").unwrap()),
            id: None,
            type_: message::MessageType::Chat,
            payloads: vec!(
                message::MessagePayloadType::Parsed(message::MessagePayload::Body("Hello world!".to_owned())),
            ),
        };
        let elem2 = message::serialise(&message);
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_attention() {
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        let message = message::parse_message(&elem).unwrap();
        let elem2 = message::serialise(&message);
        assert_eq!(elem, elem2);
    }
}
