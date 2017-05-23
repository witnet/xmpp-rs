// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Replace {
    pub id: String,
}

impl<'a> TryFrom<&'a Element> for Replace {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Replace, Error> {
        if !elem.is("replace", ns::MESSAGE_CORRECT) {
            return Err(Error::ParseError("This is not a replace element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in replace element."));
        }
        let id = get_attr!(elem, "id", required);
        Ok(Replace { id })
    }
}

impl<'a> Into<Element> for &'a Replace {
    fn into(self) -> Element {
        Element::builder("replace")
                .ns(ns::MESSAGE_CORRECT)
                .attr("id", self.id.clone())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        Replace::try_from(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>".parse().unwrap();
        let error = Replace::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Replace::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        let replace = Replace { id: String::from("coucou") };
        let elem2 = (&replace).into();
        assert_eq!(elem, elem2);
    }
}
