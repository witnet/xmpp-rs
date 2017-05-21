// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

type Date = String;

#[derive(Debug, Clone)]
pub struct Idle {
    pub since: Date,
}

impl<'a> TryFrom<&'a Element> for Idle {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Idle, Error> {
        if !elem.is("idle", ns::IDLE) {
            return Err(Error::ParseError("This is not an idle element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in idle element."));
        }
        let since = get_attr!(elem, "since", required);
        Ok(Idle { since: since })
    }
}

impl<'a> Into<Element> for &'a Idle {
    fn into(self) -> Element {
        Element::builder("idle")
                .ns(ns::IDLE)
                .attr("since", self.since.clone())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>".parse().unwrap();
        Idle::try_from(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'><coucou/></idle>".parse().unwrap();
        let error = Idle::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in idle element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'/>".parse().unwrap();
        let error = Idle::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'since' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>".parse().unwrap();
        let idle = Idle { since: Date::from("2017-05-21T20:19:55+01:00") };
        let elem2 = (&idle).into();
        assert_eq!(elem, elem2);
    }
}
