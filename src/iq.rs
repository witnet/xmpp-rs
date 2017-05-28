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

use stanza_error::StanzaError;
use roster::Roster;
use disco::Disco;
use ibb::IBB;
use jingle::Jingle;
use ping::Ping;
use mam::{Query as MamQuery, Fin as MamFin, Prefs as MamPrefs};

/// Lists every known payload of a `<iq/>`.
#[derive(Debug, Clone)]
pub enum IqPayload {
    Roster(Roster),
    Disco(Disco),
    IBB(IBB),
    Jingle(Jingle),
    Ping(Ping),
    MamQuery(MamQuery),
    MamFin(MamFin),
    MamPrefs(MamPrefs),

    Unknown(Element),
}

impl TryFrom<Element> for IqPayload {
    type Error = Error;

    fn try_from(elem: Element) -> Result<IqPayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            // RFC-6121
            ("query", ns::ROSTER) => IqPayload::Roster(Roster::try_from(elem)?),

            // XEP-0030
            ("query", ns::DISCO_INFO) => IqPayload::Disco(Disco::try_from(elem)?),

            // XEP-0047
            ("open", ns::IBB)
          | ("data", ns::IBB)
          | ("close", ns::IBB) => IqPayload::IBB(IBB::try_from(elem)?),

            // XEP-0166
            ("jingle", ns::JINGLE) => IqPayload::Jingle(Jingle::try_from(elem)?),

            // XEP-0199
            ("ping", ns::PING) => IqPayload::Ping(Ping::try_from(elem)?),

            // XEP-0313
            ("query", ns::MAM) => IqPayload::MamQuery(MamQuery::try_from(elem)?),
            ("fin", ns::MAM) => IqPayload::MamFin(MamFin::try_from(elem)?),
            ("prefs", ns::MAM) => IqPayload::MamPrefs(MamPrefs::try_from(elem)?),

            _ => IqPayload::Unknown(elem),
        })
    }
}

#[derive(Debug, Clone)]
pub enum IqType {
    Get(Element),
    Set(Element),
    Result(Option<Element>),
    Error(StanzaError),
}

impl<'a> IntoAttributeValue for &'a IqType {
    fn into_attribute_value(self) -> Option<String> {
        Some(match *self {
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

impl TryFrom<Element> for Iq {
    type Error = Error;

    fn try_from(root: Element) -> Result<Iq, Error> {
        if !root.is("iq", ns::JABBER_CLIENT) {
            return Err(Error::ParseError("This is not an iq element."));
        }
        let from = get_attr!(root, "from", optional);
        let to = get_attr!(root, "to", optional);
        let id = get_attr!(root, "id", optional);
        let type_: String = get_attr!(root, "type", required);

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
                    error_payload = Some(StanzaError::try_from(elem.clone())?);
                } else if root.children().collect::<Vec<_>>().len() != 2 {
                    return Err(Error::ParseError("Wrong number of children in iq element."));
                }
            } else {
                payload = Some(elem.clone());
            }
        }

        let type_ = if type_ == "get" {
            if let Some(payload) = payload {
                IqType::Get(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else if type_ == "set" {
            if let Some(payload) = payload {
                IqType::Set(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else if type_ == "result" {
            if let Some(payload) = payload {
                IqType::Result(Some(payload))
            } else {
                IqType::Result(None)
            }
        } else if type_ == "error" {
            if let Some(payload) = error_payload {
                IqType::Error(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else {
            return Err(Error::ParseError("Unknown iq type."));
        };

        Ok(Iq {
            from: from,
            to: to,
            id: id,
            payload: type_,
        })
    }
}

impl Into<Element> for IqPayload {
    fn into(self) -> Element {
        match self {
            IqPayload::Roster(roster) => roster.into(),
            IqPayload::Disco(disco) => disco.into(),
            IqPayload::IBB(ibb) => ibb.into(),
            IqPayload::Jingle(jingle) => jingle.into(),
            IqPayload::Ping(ping) => ping.into(),
            IqPayload::MamQuery(query) => query.into(),
            IqPayload::MamFin(fin) => fin.into(),
            IqPayload::MamPrefs(prefs) => prefs.into(),

            IqPayload::Unknown(elem) => elem,
        }
    }
}

impl Into<Element> for Iq {
    fn into(self) -> Element {
        let mut stanza = Element::builder("iq")
                                 .ns(ns::JABBER_CLIENT)
                                 .attr("from", self.from.and_then(|value| Some(String::from(value))))
                                 .attr("to", self.to.and_then(|value| Some(String::from(value))))
                                 .attr("id", self.id)
                                 .attr("type", &self.payload)
                                 .build();
        let elem = match self.payload {
            IqType::Get(elem)
          | IqType::Set(elem)
          | IqType::Result(Some(elem)) => elem,
            IqType::Error(error) => error.into(),
            IqType::Result(None) => return stanza,
        };
        stanza.append_child(elem);
        stanza
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stanza_error::{ErrorType, DefinedCondition};

    #[test]
    fn test_require_type() {
        let elem: Element = "<iq xmlns='jabber:client'/>".parse().unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");
    }

    #[test]
    fn test_get() {
        let elem: Element = "<iq xmlns='jabber:client' type='get'>
            <foo/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<foo xmlns='jabber:client'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Get(element) => element == query,
            _ => false
        });
    }

    #[test]
    fn test_set() {
        let elem: Element = "<iq xmlns='jabber:client' type='set'>
            <vCard xmlns='vcard-temp'/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let vcard: Element = "<vCard xmlns='vcard-temp'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Set(element) => element == vcard,
            _ => false
        });
    }

    #[test]
    fn test_result_empty() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Result(None) => true,
            _ => false,
        });
    }

    #[test]
    fn test_result() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Result(Some(element)) => element == query,
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
        let iq = Iq::try_from(elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        match iq.payload {
            IqType::Error(error) => {
                assert_eq!(error.type_, ErrorType::Cancel);
                assert_eq!(error.by, None);
                assert_eq!(error.defined_condition, DefinedCondition::ServiceUnavailable);
                assert_eq!(error.texts.len(), 0);
                assert_eq!(error.other, None);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_children_invalid() {
        let elem: Element = "<iq xmlns='jabber:client' type='error'></iq>".parse().unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Wrong number of children in iq element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        let iq2 = Iq {
            from: None,
            to: None,
            id: None,
            payload: IqType::Result(None),
        };
        let elem2 = iq2.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_disco() {
        let elem: Element = "<iq xmlns='jabber:client' type='get'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let payload = match iq.payload {
            IqType::Get(payload) => IqPayload::try_from(payload).unwrap(),
            _ => panic!(),
        };
        assert!(match payload {
            IqPayload::Disco(Disco { .. }) => true,
            _ => false,
        });
    }
}
