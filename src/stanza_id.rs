// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub enum StanzaId {
    StanzaId {
        id: String,
        by: Jid,
    },
    OriginId {
        id: String,
    },
}

impl<'a> TryFrom<&'a Element> for StanzaId {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<StanzaId, Error> {
        let is_stanza_id = elem.is("stanza-id", ns::SID);
        if !is_stanza_id && !elem.is("origin-id", ns::SID) {
            return Err(Error::ParseError("This is not a stanza-id or origin-id element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in stanza-id or origin-id element."));
        }
        let id = match elem.attr("id") {
            Some(id) => id.to_owned(),
            None => return Err(Error::ParseError("No 'id' attribute present in stanza-id or origin-id.")),
        };
        Ok(if is_stanza_id {
            let by = match elem.attr("by") {
                Some(by) => by.parse().unwrap(),
                None => return Err(Error::ParseError("No 'by' attribute present in stanza-id.")),
            };
            StanzaId::StanzaId { id, by }
        } else {
            StanzaId::OriginId { id }
        })
    }
}

impl<'a> Into<Element> for &'a StanzaId {
    fn into(self) -> Element {
        match *self {
            StanzaId::StanzaId { ref id, ref by } => {
                Element::builder("stanza-id")
                        .ns(ns::SID)
                        .attr("id", id.clone())
                        .attr("by", String::from(by.clone()))
                        .build()
            },
            StanzaId::OriginId { ref id } => {
                Element::builder("origin-id")
                        .ns(ns::SID)
                        .attr("id", id.clone())
                        .build()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>".parse().unwrap();
        let stanza_id = StanzaId::try_from(&elem).unwrap();
        if let StanzaId::StanzaId { id, by } = stanza_id {
            assert_eq!(id, String::from("coucou"));
            assert_eq!(by, Jid::from_str("coucou@coucou").unwrap());
        } else {
            panic!();
        }

        let elem: Element = "<origin-id xmlns='urn:xmpp:sid:0' id='coucou'/>".parse().unwrap();
        let stanza_id = StanzaId::try_from(&elem).unwrap();
        if let StanzaId::OriginId { id } = stanza_id {
            assert_eq!(id, String::from("coucou"));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'><coucou/></stanza-id>".parse().unwrap();
        let error = StanzaId::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in stanza-id or origin-id element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'/>".parse().unwrap();
        let error = StanzaId::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No 'id' attribute present in stanza-id or origin-id.");
    }

    #[test]
    fn test_invalid_by() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou'/>".parse().unwrap();
        let error = StanzaId::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No 'by' attribute present in stanza-id.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>".parse().unwrap();
        let stanza_id = StanzaId::StanzaId { id: String::from("coucou"), by: Jid::from_str("coucou@coucou").unwrap() };
        let elem2 = (&stanza_id).into();
        assert_eq!(elem, elem2);
    }
}
