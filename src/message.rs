// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::collections::BTreeMap;

use minidom::Element;

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
            ("error", ns::DEFAULT_NS) => MessagePayload::StanzaError(StanzaError::try_from(elem)?),

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

generate_elem_id!(Body, "body", DEFAULT_NS);
generate_elem_id!(Subject, "subject", DEFAULT_NS);
generate_elem_id!(Thread, "thread", DEFAULT_NS);

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

    fn get_best<'a, T>(map: &'a BTreeMap<Lang, T>, preferred_langs: Vec<&str>) -> Option<(Lang, &'a T)> {
        if map.is_empty() {
            return None;
        }
        for lang in preferred_langs {
            if let Some(value) = map.get(lang) {
                return Some((Lang::from(lang), value));
            }
        }
        if let Some(value) = map.get("") {
            return Some((Lang::new(), value));
        }
        map.iter().map(|(lang, value)| (lang.clone(), value)).next()
    }

    /// Returns the best matching body from a list of languages.
    ///
    /// For instance, if a message contains both an xml:lang='de', an xml:lang='fr' and an English
    /// body without an xml:lang attribute, and you pass ["fr", "en"] as your preferred languages,
    /// `Some(("fr", the_second_body))` will be returned.
    ///
    /// If no body matches, an undefined body will be returned.
    pub fn get_best_body(&self, preferred_langs: Vec<&str>) -> Option<(Lang, &Body)> {
        Message::get_best::<Body>(&self.bodies, preferred_langs)
    }

    /// Returns the best matching subject from a list of languages.
    ///
    /// For instance, if a message contains both an xml:lang='de', an xml:lang='fr' and an English
    /// subject without an xml:lang attribute, and you pass ["fr", "en"] as your preferred
    /// languages, `Some(("fr", the_second_subject))` will be returned.
    ///
    /// If no subject matches, an undefined subject will be returned.
    pub fn get_best_subject(&self, preferred_langs: Vec<&str>) -> Option<(Lang, &Subject)> {
        Message::get_best::<Subject>(&self.subjects, preferred_langs)
    }
}

impl TryFrom<Element> for Message {
    type Err = Error;

    fn try_from(root: Element) -> Result<Message, Error> {
        check_self!(root, "message", DEFAULT_NS);
        let from = get_attr!(root, "from", optional);
        let to = get_attr!(root, "to", optional);
        let id = get_attr!(root, "id", optional);
        let type_ = get_attr!(root, "type", default);
        let mut bodies = BTreeMap::new();
        let mut subjects = BTreeMap::new();
        let mut thread = None;
        let mut payloads = vec!();
        for elem in root.children() {
            if elem.is("body", ns::DEFAULT_NS) {
                check_no_children!(elem, "body");
                let lang = get_attr!(elem, "xml:lang", default);
                let body = Body(elem.text());
                if bodies.insert(lang, body).is_some() {
                    return Err(Error::ParseError("Body element present twice for the same xml:lang."));
                }
            } else if elem.is("subject", ns::DEFAULT_NS) {
                check_no_children!(elem, "subject");
                let lang = get_attr!(elem, "xml:lang", default);
                let subject = Subject(elem.text());
                if subjects.insert(lang, subject).is_some() {
                    return Err(Error::ParseError("Subject element present twice for the same xml:lang."));
                }
            } else if elem.is("thread", ns::DEFAULT_NS) {
                if thread.is_some() {
                    return Err(Error::ParseError("Thread element present twice."));
                }
                check_no_children!(elem, "thread");
                thread = Some(Thread(elem.text()));
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
                .ns(ns::DEFAULT_NS)
                .attr("from", message.from)
                .attr("to", message.to)
                .attr("id", message.id)
                .attr("type", message.type_)
                .append(message.subjects.into_iter()
                                        .map(|(lang, subject)| {
                                                 let mut subject = Element::from(subject);
                                                 subject.set_attr("xml:lang", match lang.as_ref() {
                                                     "" => None,
                                                     lang => Some(lang),
                                                 });
                                                 subject
                                             })
                                        .collect::<Vec<_>>())
                .append(message.bodies.into_iter()
                                      .map(|(lang, body)| {
                                               let mut body = Element::from(body);
                                               body.set_attr("xml:lang", match lang.as_ref() {
                                                   "" => None,
                                                   lang => Some(lang),
                                               });
                                               body
                                           })
                                      .collect::<Vec<_>>())
                .append(message.payloads)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept'/>".parse().unwrap();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.from, None);
        assert_eq!(message.to, None);
        assert_eq!(message.id, None);
        assert_eq!(message.type_, MessageType::Normal);
        assert!(message.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept'/>".parse().unwrap();
        let mut message = Message::new(None);
        message.type_ = MessageType::Normal;
        let elem2 = message.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.bodies[""], Body::from_str("Hello world!").unwrap());

        {
            let (lang, body) = message.get_best_body(vec!("en")).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        let elem2 = message.into();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn test_serialise_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let mut message = Message::new(Some(Jid::from_str("coucou@example.org").unwrap()));
        message.bodies.insert(String::from(""), Body::from_str("Hello world!").unwrap());
        let elem2 = message.into();
        assert!(elem.compare_to(&elem2));
    }

    #[test]
    fn test_subject() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.subjects[""], Subject::from_str("Hello world!").unwrap());

        {
            let (lang, subject) = message.get_best_subject(vec!("en")).unwrap();
            assert_eq!(lang, "");
            assert_eq!(subject, &Subject::from_str("Hello world!").unwrap());
        }

        let elem2 = message.into();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn get_best_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body xml:lang='de'>Hallo Welt!</body><body xml:lang='fr'>Salut le monde !</body><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let message = Message::try_from(elem).unwrap();

        // Tests basic feature.
        {
            let (lang, body) = message.get_best_body(vec!("fr")).unwrap();
            assert_eq!(lang, "fr");
            assert_eq!(body, &Body::from_str("Salut le monde !").unwrap());
        }

        // Tests order.
        {
            let (lang, body) = message.get_best_body(vec!("en", "de")).unwrap();
            assert_eq!(lang, "de");
            assert_eq!(body, &Body::from_str("Hallo Welt!").unwrap());
        }

        // Tests fallback.
        {
            let (lang, body) = message.get_best_body(vec!()).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        // Tests fallback.
        {
            let (lang, body) = message.get_best_body(vec!("ja")).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        let message = Message::new(None);

        // Tests without a body.
        assert_eq!(message.get_best_body(vec!("ja")), None);
    }

    #[test]
    fn test_attention() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }
}
