// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use minidom::{Element, IntoElements, IntoAttributeValue};
use minidom::convert::ElementEmitter;

use jid::Jid;

use error::Error;

use ns;

use delay;
use ecaps2;

#[derive(Debug, Clone, PartialEq)]
pub enum Show {
    Away,
    Chat,
    Dnd,
    Xa,
}

impl IntoElements for Show {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let elem = Element::builder(match self {
            Show::Away => "away",
            Show::Chat => "chat",
            Show::Dnd => "dnd",
            Show::Xa => "xa",
        }).build();
        emitter.append_child(elem);
    }
}

pub type Status = String;

pub type Priority = i8;

/// Lists every known payload of a `<presence/>`.
#[derive(Debug, Clone)]
pub enum PresencePayload {
    Show(Show),
    Status(Status),
    Priority(Priority),
    Delay(delay::Delay),
    ECaps2(ecaps2::ECaps2),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PresenceType {
    /// This value is not an acceptable 'type' attribute, it is only used
    /// internally to signal the absence of 'type'.
    Available,
    Error,
    Probe,
    Subscribe,
    Subscribed,
    Unavailable,
    Unsubscribe,
    Unsubscribed,
}

impl Default for PresenceType {
    fn default() -> PresenceType {
        PresenceType::Available
    }
}

impl FromStr for PresenceType {
    type Err = Error;

    fn from_str(s: &str) -> Result<PresenceType, Error> {
        Ok(match s {
            "error" => PresenceType::Error,
            "probe" => PresenceType::Probe,
            "subscribe" => PresenceType::Subscribe,
            "subscribed" => PresenceType::Subscribed,
            "unavailable" => PresenceType::Unavailable,
            "unsubscribe" => PresenceType::Unsubscribe,
            "unsubscribed" => PresenceType::Unsubscribed,

            _ => return Err(Error::ParseError("Invalid 'type' attribute on presence element.")),
        })
    }
}

impl IntoAttributeValue for PresenceType {
    fn into_attribute_value(self) -> Option<String> {
        Some(match self {
            PresenceType::Available => return None,

            PresenceType::Error => "error",
            PresenceType::Probe => "probe",
            PresenceType::Subscribe => "subscribe",
            PresenceType::Subscribed => "subscribed",
            PresenceType::Unavailable => "unavailable",
            PresenceType::Unsubscribe => "unsubscribe",
            PresenceType::Unsubscribed => "unsubscribed",
        }.to_owned())
    }
}

#[derive(Debug, Clone)]
pub enum PresencePayloadType {
    XML(Element),
    Parsed(PresencePayload),
}

#[derive(Debug, Clone)]
pub struct Presence {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub type_: PresenceType,
    pub payloads: Vec<PresencePayloadType>,
}

pub fn parse_presence(root: &Element) -> Result<Presence, Error> {
    if !root.is("presence", ns::JABBER_CLIENT) {
        return Err(Error::ParseError("This is not a presence element."));
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
        if elem.is("show", ns::JABBER_CLIENT) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in show element."));
            }
            let payload = PresencePayload::Show(match elem.text().as_ref() {
                "away" => Show::Away,
                "chat" => Show::Chat,
                "dnd" => Show::Dnd,
                "xa" => Show::Xa,

                _ => return Err(Error::ParseError("Invalid value for show.")),
            });
            payloads.push(PresencePayloadType::Parsed(payload));
        } else if elem.is("status", ns::JABBER_CLIENT) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in status element."));
            }
            let payload = PresencePayload::Status(elem.text());
            payloads.push(PresencePayloadType::Parsed(payload));
        } else if elem.is("priority", ns::JABBER_CLIENT) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in priority element."));
            }
            let priority = Priority::from_str(elem.text().as_ref())?;
            let payload = PresencePayload::Priority(priority);
            payloads.push(PresencePayloadType::Parsed(payload));
        } else {
            let payload = if let Ok(delay) = delay::parse_delay(elem) {
                Some(PresencePayload::Delay(delay))
            } else if let Ok(ecaps2) = ecaps2::parse_ecaps2(elem) {
                Some(PresencePayload::ECaps2(ecaps2))
            } else {
                None
            };
            payloads.push(match payload {
                Some(payload) => PresencePayloadType::Parsed(payload),
                None => PresencePayloadType::XML(elem.clone()),
            });
        }
    }
    Ok(Presence {
        from: from,
        to: to,
        id: id,
        type_: type_,
        payloads: payloads,
    })
}

pub fn serialise_payload(payload: &PresencePayload) -> Element {
    match *payload {
        PresencePayload::Show(ref show) => {
            Element::builder("status")
                    .ns(ns::JABBER_CLIENT)
                    .append(show.to_owned())
                    .build()
        },
        PresencePayload::Status(ref status) => {
            Element::builder("status")
                    .ns(ns::JABBER_CLIENT)
                    .append(status.to_owned())
                    .build()
        },
        PresencePayload::Priority(ref priority) => {
            Element::builder("status")
                    .ns(ns::JABBER_CLIENT)
                    .append(format!("{}", priority))
                    .build()
        },
        PresencePayload::Delay(ref delay) => delay::serialise(delay),
        PresencePayload::ECaps2(ref ecaps2) => ecaps2::serialise(ecaps2),
    }
}

pub fn serialise(presence: &Presence) -> Element {
    let mut stanza = Element::builder("presence")
                             .ns(ns::JABBER_CLIENT)
                             .attr("from", presence.from.clone().and_then(|value| Some(String::from(value))))
                             .attr("to", presence.to.clone().and_then(|value| Some(String::from(value))))
                             .attr("id", presence.id.clone())
                             .attr("type", presence.type_.clone())
                             .build();
    for child in presence.payloads.clone() {
        let elem = match child {
            PresencePayloadType::XML(elem) => elem,
            PresencePayloadType::Parsed(payload) => serialise_payload(&payload),
        };
        stanza.append_child(elem);
    }
    stanza
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use presence;
    use ns;

    #[test]
    fn test_simple() {
        let elem: Element = "<presence xmlns='jabber:client'/>".parse().unwrap();
        let presence = presence::parse_presence(&elem).unwrap();
        assert_eq!(presence.from, None);
        assert_eq!(presence.to, None);
        assert_eq!(presence.id, None);
        assert_eq!(presence.type_, presence::PresenceType::Available);
        assert!(presence.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<presence xmlns='jabber:client' type='unavailable'/>".parse().unwrap();
        let presence = presence::Presence {
            from: None,
            to: None,
            id: None,
            type_: presence::PresenceType::Unavailable,
            payloads: vec!(),
        };
        let elem2 = presence::serialise(&presence);
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_show() {
        let elem: Element = "<presence xmlns='jabber:client'><show>chat</show></presence>".parse().unwrap();
        let presence = presence::parse_presence(&elem).unwrap();
        assert_eq!(presence.payloads.len(), 1);
        match presence.payloads[0] {
            presence::PresencePayloadType::Parsed(presence::PresencePayload::Show(ref show)) => {
                assert_eq!(*show, presence::Show::Chat);
            },
            _ => panic!("Failed to parse show presence."),
        }
    }

    #[test]
    fn test_missing_show_value() {
        // "online" used to be a pretty common mistake.
        let elem: Element = "<presence xmlns='jabber:client'><show/></presence>".parse().unwrap();
        let error = presence::parse_presence(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_invalid_show() {
        // "online" used to be a pretty common mistake.
        let elem: Element = "<presence xmlns='jabber:client'><show>online</show></presence>".parse().unwrap();
        let error = presence::parse_presence(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_status() {
        let elem: Element = "<presence xmlns='jabber:client'><status xmlns='jabber:client'/></presence>".parse().unwrap();
        let presence = presence::parse_presence(&elem).unwrap();
        assert_eq!(presence.payloads.len(), 1);
        match presence.payloads[0] {
            presence::PresencePayloadType::Parsed(presence::PresencePayload::Status(ref status)) => {
                assert_eq!(*status, presence::Status::from(""));
            },
            _ => panic!("Failed to parse status presence."),
        }
    }

    #[test]
    fn test_priority() {
        let elem: Element = "<presence xmlns='jabber:client'><priority>-1</priority></presence>".parse().unwrap();
        let presence = presence::parse_presence(&elem).unwrap();
        assert_eq!(presence.payloads.len(), 1);
        match presence.payloads[0] {
            presence::PresencePayloadType::Parsed(presence::PresencePayload::Priority(ref priority)) => {
                assert_eq!(*priority, presence::Priority::from(-1i8));
            },
            _ => panic!("Failed to parse priority."),
        }
    }

    #[test]
    fn test_invalid_priority() {
        let elem: Element = "<presence xmlns='jabber:client'><priority>128</priority></presence>".parse().unwrap();
        let error = presence::parse_presence(&elem).unwrap_err();
        match error {
            Error::ParseIntError(_) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn test_unknown_child() {
        let elem: Element = "<presence xmlns='jabber:client'><test xmlns='invalid'/></presence>".parse().unwrap();
        let presence = presence::parse_presence(&elem).unwrap();
        if let presence::PresencePayloadType::XML(ref payload) = presence.payloads[0] {
            assert!(payload.is("test", "invalid"));
        } else {
            panic!("Did successfully parse an invalid element.");
        }
    }

    #[test]
    #[ignore]
    fn test_invalid_status_child() {
        let elem: Element = "<presence xmlns='jabber:client'><status xmlns='jabber:client'><coucou/></status></presence>".parse().unwrap();
        let error = presence::parse_presence(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<status xmlns='jabber:client' coucou=''/>".parse().unwrap();
        let error = presence::parse_presence(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in status element.");
    }

    #[test]
    fn test_serialise_status() {
        let status = presence::Status::from("Hello world!");
        let payloads = vec!(presence::PresencePayloadType::Parsed(presence::PresencePayload::Status(status)));
        let presence = presence::Presence {
            from: None,
            to: None,
            id: None,
            type_: presence::PresenceType::Unavailable,
            payloads: payloads,
        };
        let elem = presence::serialise(&presence);
        assert!(elem.is("presence", ns::JABBER_CLIENT));
        assert!(elem.children().collect::<Vec<_>>()[0].is("status", ns::JABBER_CLIENT));
    }
}
