// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Replace {
    pub id: String,
}

impl TryFrom<Element> for Replace {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Replace, Error> {
        check_self!(elem, "replace", ns::MESSAGE_CORRECT);
        check_no_children!(elem, "replace");
        check_no_unknown_attributes!(elem, "replace", ["id"]);
        let id = get_attr!(elem, "id", required);
        Ok(Replace { id })
    }
}

impl From<Replace> for Element {
    fn from(replace: Replace) -> Element {
        Element::builder("replace")
                .ns(ns::MESSAGE_CORRECT)
                .attr("id", replace.id)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        Replace::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' coucou=''/>".parse().unwrap();
        let error = Replace::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in replace element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>".parse().unwrap();
        let error = Replace::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Replace::try_from(elem).unwrap_err();
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
        let elem2 = replace.into();
        assert_eq!(elem, elem2);
    }
}
