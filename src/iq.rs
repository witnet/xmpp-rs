// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;
use minidom::IntoAttributeValue;

use jid::Jid;

use error::Error;

use ns;

use stanza_error;
use disco;
use ibb;
use jingle::Jingle;
use ping;

/// Lists every known payload of a `<iq/>`.
#[derive(Debug, Clone)]
pub enum IqPayload {
    Disco(disco::Disco),
    IBB(ibb::IBB),
    Jingle(Jingle),
    Ping(ping::Ping),
}

#[derive(Debug, Clone)]
pub enum IqPayloadType {
    XML(Element),
    Parsed(IqPayload),
}

#[derive(Debug, Clone)]
pub enum IqType {
    Get(IqPayloadType),
    Set(IqPayloadType),
    Result(Option<IqPayloadType>),
    Error(stanza_error::StanzaError),
}

impl IntoAttributeValue for IqType {
    fn into_attribute_value(self) -> Option<String> {
        Some(match self {
            IqType::Get(_) => "get",
            IqType::Set(_) => "set",
            IqType::Result(_) => "result",
            IqType::Error(_) => "error",
        }.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct Iq {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub payload: IqType,
}

pub fn parse_iq(root: &Element) -> Result<Iq, Error> {
    if !root.is("iq", ns::JABBER_CLIENT) {
        return Err(Error::ParseError("This is not an iq element."));
    }
    let from = root.attr("from")
        .and_then(|value| value.parse().ok());
    let to = root.attr("to")
        .and_then(|value| value.parse().ok());
    let id = root.attr("id")
        .and_then(|value| value.parse().ok());
    let type_ = match root.attr("type") {
        Some(type_) => type_,
        None => return Err(Error::ParseError("Iq element requires a 'type' attribute.")),
    };

    let mut payload = None;
    let mut error_payload = None;
    for elem in root.children() {
        if payload.is_some() {
            return Err(Error::ParseError("Wrong number of children in iq element."));
        }
        if type_ == "error" {
            if elem.is("error", ns::JABBER_CLIENT) {
                if error_payload.is_some() {
                    return Err(Error::ParseError("Wrong number of children in iq element."));
                }
                error_payload = Some(stanza_error::parse_stanza_error(elem)?);
            } else if root.children().collect::<Vec<_>>().len() != 2 {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else {
            let parsed_payload = if let Ok(disco) = disco::parse_disco(elem) {
                Some(IqPayload::Disco(disco))
            } else if let Ok(ibb) = ibb::parse_ibb(elem) {
                Some(IqPayload::IBB(ibb))
            } else if let Ok(jingle) = Jingle::try_from(elem) {
                Some(IqPayload::Jingle(jingle))
            } else if let Ok(ping) = ping::parse_ping(elem) {
                Some(IqPayload::Ping(ping))
            } else {
                None
            };

            payload = match parsed_payload {
                Some(payload) => Some(IqPayloadType::Parsed(payload)),
                None => Some(IqPayloadType::XML(elem.clone())),
            };
        }
    }

    let type_ = if type_ == "get" {
        if let Some(payload) = payload.clone() {
            IqType::Get(payload.clone())
        } else {
            return Err(Error::ParseError("Wrong number of children in iq element."));
        }
    } else if type_ == "set" {
        if let Some(payload) = payload.clone() {
            IqType::Set(payload.clone())
        } else {
            return Err(Error::ParseError("Wrong number of children in iq element."));
        }
    } else if type_ == "result" {
        if let Some(payload) = payload.clone() {
            IqType::Result(Some(payload.clone()))
        } else {
            IqType::Result(None)
        }
    } else if type_ == "error" {
        if let Some(payload) = error_payload.clone() {
            IqType::Error(payload.clone())
        } else {
            return Err(Error::ParseError("Wrong number of children in iq element."));
        }
    } else {
        panic!()
    };

    Ok(Iq {
        from: from,
        to: to,
        id: id,
        payload: type_,
    })
}

pub fn serialise_payload(payload: &IqPayload) -> Element {
    match *payload {
        IqPayload::Disco(ref disco) => disco::serialise_disco(disco),
        IqPayload::IBB(ref ibb) => ibb::serialise(ibb),
        IqPayload::Jingle(ref jingle) => jingle.into(),
        IqPayload::Ping(_) => ping::serialise_ping(),
    }
}

pub fn serialise(iq: &Iq) -> Element {
    let mut stanza = Element::builder("iq")
                             .ns(ns::JABBER_CLIENT)
                             .attr("from", iq.from.clone().and_then(|value| Some(String::from(value))))
                             .attr("to", iq.to.clone().and_then(|value| Some(String::from(value))))
                             .attr("id", iq.id.clone())
                             .attr("type", iq.payload.clone())
                             .build();
    let elem = match iq.payload.clone() {
        IqType::Get(IqPayloadType::XML(elem))
      | IqType::Set(IqPayloadType::XML(elem))
      | IqType::Result(Some(IqPayloadType::XML(elem))) => elem,
        IqType::Error(error) => stanza_error::serialise(&error),
        IqType::Get(IqPayloadType::Parsed(payload))
      | IqType::Set(IqPayloadType::Parsed(payload))
      | IqType::Result(Some(IqPayloadType::Parsed(payload))) => serialise_payload(&payload),
        IqType::Result(None) => return stanza,
    };
    stanza.append_child(elem);
    stanza
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use iq;
    use stanza_error;
    use disco;

    #[test]
    fn test_require_type() {
        let elem: Element = "<iq xmlns='jabber:client'/>".parse().unwrap();
        let error = iq::parse_iq(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Iq element requires a 'type' attribute.");
    }

    #[test]
    fn test_get() {
        let elem: Element = "<iq xmlns='jabber:client' type='get'>
            <foo/>
        </iq>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        let query: Element = "<foo xmlns='jabber:client'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            iq::IqType::Get(iq::IqPayloadType::XML(element)) => element == query,
            _ => false
        });
    }

    #[test]
    fn test_set() {
        let elem: Element = "<iq xmlns='jabber:client' type='set'>
            <vCard xmlns='vcard-temp'/>
        </iq>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        let vcard: Element = "<vCard xmlns='vcard-temp'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            iq::IqType::Set(iq::IqPayloadType::XML(element)) => element == vcard,
            _ => false
        });
    }

    #[test]
    fn test_result_empty() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            iq::IqType::Result(None) => true,
            _ => false,
        });
    }

    #[test]
    fn test_result() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        let query: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            iq::IqType::Result(Some(iq::IqPayloadType::XML(element))) => element == query,
            _ => false,
        });
    }

    #[test]
    fn test_error() {
        let elem: Element = "<iq xmlns='jabber:client' type='error'>
            <ping xmlns='urn:xmpp:ping'/>
            <error type='cancel'>
                <service-unavailable xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>
            </error>
        </iq>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        match iq.payload {
            iq::IqType::Error(error) => {
                assert_eq!(error.type_, stanza_error::ErrorType::Cancel);
                assert_eq!(error.by, None);
                assert_eq!(error.defined_condition, stanza_error::DefinedCondition::ServiceUnavailable);
                assert_eq!(error.texts.len(), 0);
                assert_eq!(error.other, None);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_children_invalid() {
        let elem: Element = "<iq xmlns='jabber:client' type='error'></iq>".parse().unwrap();
        let error = iq::parse_iq(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Wrong number of children in iq element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        let iq2 = iq::Iq {
            from: None,
            to: None,
            id: None,
            payload: iq::IqType::Result(None),
        };
        let elem2 = iq::serialise(&iq2);
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_disco() {
        let elem: Element = "<iq xmlns='jabber:client' type='get'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        let iq = iq::parse_iq(&elem).unwrap();
        assert!(match iq.payload {
            iq::IqType::Get(iq::IqPayloadType::Parsed(iq::IqPayload::Disco(disco::Disco { .. }))) => true,
            _ => false,
        });
    }
}
