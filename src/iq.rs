// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use minidom::IntoAttributeValue;

use jid::Jid;

use error::Error;

use ns;

use stanza_error::StanzaError;
use roster::Roster;
use disco::{DiscoInfoResult, DiscoInfoQuery};
use ibb::{Open as IbbOpen, Data as IbbData, Close as IbbClose};
use jingle::Jingle;
use ping::Ping;
use mam::{Query as MamQuery, Fin as MamFin, Prefs as MamPrefs};

/// Lists every known payload of an `<iq type='get'/>`.
#[derive(Debug, Clone)]
pub enum IqGetPayload {
    Roster(Roster),
    DiscoInfo(DiscoInfoQuery),
    Ping(Ping),
    MamQuery(MamQuery),
    MamPrefs(MamPrefs),

    Unknown(Element),
}

/// Lists every known payload of an `<iq type='set'/>`.
#[derive(Debug, Clone)]
pub enum IqSetPayload {
    Roster(Roster),
    IbbOpen(IbbOpen),
    IbbData(IbbData),
    IbbClose(IbbClose),
    Jingle(Jingle),
    MamQuery(MamQuery),
    MamPrefs(MamPrefs),

    Unknown(Element),
}

/// Lists every known payload of an `<iq type='result'/>`.
#[derive(Debug, Clone)]
pub enum IqResultPayload {
    Roster(Roster),
    DiscoInfo(DiscoInfoResult),
    MamQuery(MamQuery),
    MamFin(MamFin),
    MamPrefs(MamPrefs),

    Unknown(Element),
}

impl TryFrom<Element> for IqGetPayload {
    type Err = Error;

    fn try_from(elem: Element) -> Result<IqGetPayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            // RFC-6121
            ("query", ns::ROSTER) => IqGetPayload::Roster(Roster::try_from(elem)?),

            // XEP-0030
            ("query", ns::DISCO_INFO) => IqGetPayload::DiscoInfo(DiscoInfoQuery::try_from(elem)?),

            // XEP-0199
            ("ping", ns::PING) => IqGetPayload::Ping(Ping::try_from(elem)?),

            // XEP-0313
            ("query", ns::MAM) => IqGetPayload::MamQuery(MamQuery::try_from(elem)?),
            ("prefs", ns::MAM) => IqGetPayload::MamPrefs(MamPrefs::try_from(elem)?),

            _ => IqGetPayload::Unknown(elem),
        })
    }
}

impl From<IqGetPayload> for Element {
    fn from(payload: IqGetPayload) -> Element {
        match payload {
            IqGetPayload::Roster(roster) => roster.into(),
            IqGetPayload::DiscoInfo(disco) => disco.into(),
            IqGetPayload::Ping(ping) => ping.into(),
            IqGetPayload::MamQuery(query) => query.into(),
            IqGetPayload::MamPrefs(prefs) => prefs.into(),

            IqGetPayload::Unknown(elem) => elem,
        }
    }
}

impl TryFrom<Element> for IqSetPayload {
    type Err = Error;

    fn try_from(elem: Element) -> Result<IqSetPayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            // RFC-6121
            ("query", ns::ROSTER) => IqSetPayload::Roster(Roster::try_from(elem)?),

            // XEP-0047
            ("open", ns::IBB) => IqSetPayload::IbbOpen(IbbOpen::try_from(elem)?),
            ("data", ns::IBB) => IqSetPayload::IbbData(IbbData::try_from(elem)?),
            ("close", ns::IBB) => IqSetPayload::IbbClose(IbbClose::try_from(elem)?),

            // XEP-0166
            ("jingle", ns::JINGLE) => IqSetPayload::Jingle(Jingle::try_from(elem)?),

            // XEP-0313
            ("query", ns::MAM) => IqSetPayload::MamQuery(MamQuery::try_from(elem)?),
            ("prefs", ns::MAM) => IqSetPayload::MamPrefs(MamPrefs::try_from(elem)?),

            _ => IqSetPayload::Unknown(elem),
        })
    }
}

impl From<IqSetPayload> for Element {
    fn from(payload: IqSetPayload) -> Element {
        match payload {
            IqSetPayload::Roster(roster) => roster.into(),
            IqSetPayload::IbbOpen(open) => open.into(),
            IqSetPayload::IbbData(data) => data.into(),
            IqSetPayload::IbbClose(close) => close.into(),
            IqSetPayload::Jingle(jingle) => jingle.into(),
            IqSetPayload::MamQuery(query) => query.into(),
            IqSetPayload::MamPrefs(prefs) => prefs.into(),

            IqSetPayload::Unknown(elem) => elem,
        }
    }
}

impl TryFrom<Element> for IqResultPayload {
    type Err = Error;

    fn try_from(elem: Element) -> Result<IqResultPayload, Error> {
        Ok(match (elem.name().as_ref(), elem.ns().unwrap().as_ref()) {
            // RFC-6121
            ("query", ns::ROSTER) => IqResultPayload::Roster(Roster::try_from(elem)?),

            // XEP-0030
            ("query", ns::DISCO_INFO) => IqResultPayload::DiscoInfo(DiscoInfoResult::try_from(elem)?),

            // XEP-0313
            ("query", ns::MAM) => IqResultPayload::MamQuery(MamQuery::try_from(elem)?),
            ("fin", ns::MAM) => IqResultPayload::MamFin(MamFin::try_from(elem)?),
            ("prefs", ns::MAM) => IqResultPayload::MamPrefs(MamPrefs::try_from(elem)?),

            _ => IqResultPayload::Unknown(elem),
        })
    }
}

impl From<IqResultPayload> for Element {
    fn from(payload: IqResultPayload) -> Element {
        match payload {
            IqResultPayload::Roster(roster) => roster.into(),
            IqResultPayload::DiscoInfo(disco) => disco.into(),
            IqResultPayload::MamQuery(query) => query.into(),
            IqResultPayload::MamFin(fin) => fin.into(),
            IqResultPayload::MamPrefs(prefs) => prefs.into(),

            IqResultPayload::Unknown(elem) => elem,
        }
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

/// The main structure representing the `<iq/>` stanza.
#[derive(Debug, Clone)]
pub struct Iq {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<String>,
    pub payload: IqType,
}

impl TryFrom<Element> for Iq {
    type Err = Error;

    fn try_from(root: Element) -> Result<Iq, Error> {
        if !root.is("iq", ns::DEFAULT_NS) {
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
                if elem.is("error", ns::DEFAULT_NS) {
                    if error_payload.is_some() {
                        return Err(Error::ParseError("Wrong number of children in iq element."));
                    }
                    error_payload = Some(StanzaError::try_from(elem.clone())?);
                } else if root.children().count() != 2 {
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

impl From<Iq> for Element {
    fn from(iq: Iq) -> Element {
        let mut stanza = Element::builder("iq")
                                 .ns(ns::DEFAULT_NS)
                                 .attr("from", iq.from)
                                 .attr("to", iq.to)
                                 .attr("id", iq.id)
                                 .attr("type", &iq.payload)
                                 .build();
        let elem = match iq.payload {
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
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_require_type() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept'/>".parse().unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");
    }

    #[test]
    fn test_get() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='get'>
            <foo xmlns='bar'/>
        </iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='get'>
            <foo xmlns='bar'/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<foo xmlns='bar'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Get(element) => element.compare_to(&query),
            _ => false
        });
    }

    #[test]
    fn test_set() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='set'>
            <vCard xmlns='vcard-temp'/>
        </iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='set'>
            <vCard xmlns='vcard-temp'/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let vcard: Element = "<vCard xmlns='vcard-temp'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Set(element) => element.compare_to(&vcard),
            _ => false
        });
    }

    #[test]
    fn test_result_empty() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result'/>".parse().unwrap();
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
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, None);
        assert!(match iq.payload {
            IqType::Result(Some(element)) => element.compare_to(&query),
            _ => false,
        });
    }

    #[test]
    fn test_error() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='error'>
            <ping xmlns='urn:xmpp:ping'/>
            <error type='cancel'>
                <service-unavailable xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>
            </error>
        </iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='error'>
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
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='error'></iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='error'></iq>".parse().unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Wrong number of children in iq element.");
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result'/>".parse().unwrap();
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
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='get'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='get'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let payload = match iq.payload {
            IqType::Get(payload) => IqGetPayload::try_from(payload).unwrap(),
            _ => panic!(),
        };
        assert!(match payload {
            IqGetPayload::DiscoInfo(DiscoInfoQuery { .. }) => true,
            _ => false,
        });
    }
}
