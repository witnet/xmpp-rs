// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;
use std::collections::BTreeMap;

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};

use jid::Jid;

use error::Error;

use ns;

use stanza_error::StanzaError;
use muc::Muc;
use caps::Caps;
use delay::Delay;
use idle::Idle;
use ecaps2::ECaps2;

#[derive(Debug, Clone, PartialEq)]
pub enum Show {
    None,
    Away,
    Chat,
    Dnd,
    Xa,
}

impl Default for Show {
    fn default() -> Show {
        Show::None
    }
}

impl FromStr for Show {
    type Err = Error;

    fn from_str(s: &str) -> Result<Show, Error> {
        Ok(match s {
            "away" => Show::Away,
            "chat" => Show::Chat,
            "dnd" => Show::Dnd,
            "xa" => Show::Xa,

            _ => return Err(Error::ParseError("Invalid value for show.")),
        })
    }
}

impl IntoElements for Show {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        if self == Show::None {
            return;
        }
        emitter.append_child(
            Element::builder("show")
                    .append(match self {
                         Show::None => unreachable!(),
                         Show::Away => Some("away"),
                         Show::Chat => Some("chat"),
                         Show::Dnd => Some("dnd"),
                         Show::Xa => Some("xa"),
                     })
                    .build())
    }
}

pub type Lang = String;
pub type Status = String;

pub type Priority = i8;

/// Lists every known payload of a `<presence/>`.
#[derive(Debug, Clone)]
pub enum PresencePayload {
    StanzaError(StanzaError),
    Muc(Muc),
    Caps(Caps),
    Delay(Delay),
    Idle(Idle),
    ECaps2(ECaps2),

    Unknown(Element),
}

impl TryFrom<Element> for PresencePayload {
    type Err = Error;

    fn try_from(elem: Element) -> Result<PresencePayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            ("error", ns::DEFAULT_NS) => PresencePayload::StanzaError(StanzaError::try_from(elem)?),

            // XEP-0045
            ("x", ns::MUC) => PresencePayload::Muc(Muc::try_from(elem)?),

            // XEP-0115
            ("c", ns::CAPS) => PresencePayload::Caps(Caps::try_from(elem)?),

            // XEP-0203
            ("delay", ns::DELAY) => PresencePayload::Delay(Delay::try_from(elem)?),

            // XEP-0319
            ("idle", ns::IDLE) => PresencePayload::Idle(Idle::try_from(elem)?),

            // XEP-0390
            ("c", ns::ECAPS2) => PresencePayload::ECaps2(ECaps2::try_from(elem)?),

            _ => PresencePayload::Unknown(elem),
        })
    }
}

impl From<PresencePayload> for Element {
    fn from(payload: PresencePayload) -> Element {
        match payload {
            PresencePayload::StanzaError(stanza_error) => stanza_error.into(),
            PresencePayload::Muc(muc) => muc.into(),
            PresencePayload::Caps(caps) => caps.into(),
            PresencePayload::Delay(delay) => delay.into(),
            PresencePayload::Idle(idle) => idle.into(),
            PresencePayload::ECaps2(ecaps2) => ecaps2.into(),

            PresencePayload::Unknown(elem) => elem,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// This value is not an acceptable 'type' attribute, it is only used
    /// internally to signal the absence of 'type'.
    None,
    Error,
    Probe,
    Subscribe,
    Subscribed,
    Unavailable,
    Unsubscribe,
    Unsubscribed,
}

impl Default for Type {
    fn default() -> Type {
        Type::None
    }
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Type, Error> {
        Ok(match s {
            "error" => Type::Error,
            "probe" => Type::Probe,
            "subscribe" => Type::Subscribe,
            "subscribed" => Type::Subscribed,
            "unavailable" => Type::Unavailable,
            "unsubscribe" => Type::Unsubscribe,
            "unsubscribed" => Type::Unsubscribed,

            _ => return Err(Error::ParseError("Invalid 'type' attribute on presence element.")),
        })
    }
}

impl IntoAttributeValue for Type {
    fn into_attribute_value(self) -> Option<String> {
        Some(match self {
            Type::None => return None,

            Type::Error => "error",
            Type::Probe => "probe",
            Type::Subscribe => "subscribe",
            Type::Subscribed => "subscribed",
            Type::Unavailable => "unavailable",
            Type::Unsubscribe => "unsubscribe",
            Type::Unsubscribed => "unsubscribed",
        }.to_owned())
    }
}

/// The main structure representing the `<presence/>` stanza.
#[derive(Debug, Clone)]
pub struct Presence {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: Type,
    pub show: Show,
    pub statuses: BTreeMap<Lang, Status>,
    pub priority: Priority,
    pub payloads: Vec<Element>,
}

impl Presence {
    pub fn new(type_: Type) -> Presence {
        Presence {
            from: None,
            to: None,
            id: None,
            type_: type_,
            show: Show::None,
            statuses: BTreeMap::new(),
            priority: 0i8,
            payloads: vec!(),
        }
    }

    pub fn with_from(mut self, from: Option<Jid>) -> Presence {
        self.from = from;
        self
    }

    pub fn with_to(mut self, to: Option<Jid>) -> Presence {
        self.to = to;
        self
    }

    pub fn with_id(mut self, id: Option<String>) -> Presence {
        self.id = id;
        self
    }

    pub fn with_show(mut self, show: Show) -> Presence {
        self.show = show;
        self
    }

    pub fn with_priority(mut self, priority: i8) -> Presence {
        self.priority = priority;
        self
    }

    pub fn with_payloads(mut self, payloads: Vec<Element>) -> Presence {
        self.payloads = payloads;
        self
    }
}

impl TryFrom<Element> for Presence {
    type Err = Error;

    fn try_from(root: Element) -> Result<Presence, Error> {
        if !root.is("presence", ns::DEFAULT_NS) {
            return Err(Error::ParseError("This is not a presence element."));
        }
        let mut show = None;
        let mut priority = None;
        let mut presence = Presence {
            from: get_attr!(root, "from", optional),
            to: get_attr!(root, "to", optional),
            id: get_attr!(root, "id", optional),
            type_: get_attr!(root, "type", default),
            show: Show::None,
            statuses: BTreeMap::new(),
            priority: 0i8,
            payloads: vec!(),
        };
        for elem in root.children() {
            if elem.is("show", ns::DEFAULT_NS) {
                if show.is_some() {
                    return Err(Error::ParseError("More than one show element in a presence."));
                }
                check_no_attributes!(elem, "show");
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in show element."));
                }
                show = Some(Show::from_str(elem.text().as_ref())?);
            } else if elem.is("status", ns::DEFAULT_NS) {
                check_no_unknown_attributes!(elem, "status", ["xml:lang"]);
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in status element."));
                }
                let lang = get_attr!(elem, "xml:lang", default);
                if presence.statuses.insert(lang, elem.text()).is_some() {
                    return Err(Error::ParseError("Status element present twice for the same xml:lang."));
                }
            } else if elem.is("priority", ns::DEFAULT_NS) {
                if priority.is_some() {
                    return Err(Error::ParseError("More than one priority element in a presence."));
                }
                check_no_attributes!(elem, "status");
                for _ in elem.children() {
                    return Err(Error::ParseError("Unknown child in priority element."));
                }
                priority = Some(Priority::from_str(elem.text().as_ref())?);
            } else {
                presence.payloads.push(elem.clone());
            }
        }
        if let Some(show) = show {
            presence.show = show;
        }
        if let Some(priority) = priority {
            presence.priority = priority;
        }
        Ok(presence)
    }
}

impl From<Presence> for Element {
    fn from(presence: Presence) -> Element {
        Element::builder("presence")
                .ns(ns::DEFAULT_NS)
                .attr("from", presence.from)
                .attr("to", presence.to)
                .attr("id", presence.id)
                .attr("type", presence.type_)
                .append(presence.show)
                .append(presence.statuses.into_iter().map(|(lang, status)| {
                     Element::builder("status")
                             .attr("xml:lang", match lang.as_ref() {
                                  "" => None,
                                  lang => Some(lang),
                              })
                             .append(status)
                             .build()
                 }).collect::<Vec<_>>())
                .append(if presence.priority == 0 { None } else { Some(format!("{}", presence.priority)) })
                .append(presence.payloads)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'/>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.from, None);
        assert_eq!(presence.to, None);
        assert_eq!(presence.id, None);
        assert_eq!(presence.type_, Type::None);
        assert!(presence.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client' type='unavailable'/>/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept' type='unavailable'/>/>".parse().unwrap();
        let presence = Presence::new(Type::Unavailable);
        let elem2 = presence.into();
        assert!(elem.compare_to(&elem2));
    }

    #[test]
    fn test_show() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show>chat</show></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><show>chat</show></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.show, Show::Chat);
    }

    #[test]
    fn test_missing_show_value() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show/></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><show/></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_invalid_show() {
        // "online" used to be a pretty common mistake.
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show>online</show></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><show>online</show></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_empty_status() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status/></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status/></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 1);
        assert_eq!(presence.statuses[""], "");
    }

    #[test]
    fn test_status() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status>Here!</status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status>Here!</status></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 1);
        assert_eq!(presence.statuses[""], "Here!");
    }

    #[test]
    fn test_multiple_statuses() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 2);
        assert_eq!(presence.statuses[""], "Here!");
        assert_eq!(presence.statuses["fr"], "Là!");
    }

    #[test]
    fn test_invalid_multiple_statuses() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status xml:lang='fr'>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status xml:lang='fr'>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Status element present twice for the same xml:lang.");
    }

    #[test]
    fn test_priority() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><priority>-1</priority></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><priority>-1</priority></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.priority, -1i8);
    }

    #[test]
    fn test_invalid_priority() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><priority>128</priority></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><priority>128</priority></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        match error {
            Error::ParseIntError(_) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn test_unknown_child() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><test xmlns='invalid'/></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><test xmlns='invalid'/></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        let payload = &presence.payloads[0];
        assert!(payload.is("test", "invalid"));
    }

    #[test]
    fn test_invalid_status_child() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status><coucou/></status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status><coucou/></status></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    fn test_invalid_attribute() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status coucou=''/></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status coucou=''/></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in status element.");
    }

    #[test]
    fn test_serialise_status() {
        let status = Status::from("Hello world!");
        let mut presence = Presence::new(Type::Unavailable);
        presence.statuses.insert(String::from(""), status);
        let elem: Element = presence.into();
        assert!(elem.is("presence", ns::DEFAULT_NS));
        assert!(elem.children().next().unwrap().is("status", ns::DEFAULT_NS));
    }
}
