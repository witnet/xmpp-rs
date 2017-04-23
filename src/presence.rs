use std::str::FromStr;

use minidom::Element;
use minidom::IntoAttributeValue;

use jid::Jid;

use error::Error;

use ns;

use status;
use delay;
use ecaps2;

/// Lists every known payload of a `<presence/>`.
#[derive(Debug, Clone)]
pub enum PresencePayload {
    Status(status::Status),
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
        let payload = if let Ok(status) = status::parse_status(elem) {
            Some(PresencePayload::Status(status))
        } else if let Ok(delay) = delay::parse_delay(elem) {
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
        PresencePayload::Status(ref status) => status::serialise(status),
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
    use presence;

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
        let presence = presence::parse_presence(&elem).unwrap();
        let presence2 = presence::Presence {
            from: None,
            to: None,
            id: None,
            type_: presence::PresenceType::Unavailable,
            payloads: vec!(),
        };
        let elem2 = presence::serialise(&presence2);
        assert_eq!(elem, elem2);
        println!("{:#?}", presence);
    }
}
