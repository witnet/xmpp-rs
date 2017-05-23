// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Ping;

impl TryFrom<Element> for Ping {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Ping, Error> {
        if !elem.is("ping", ns::PING) {
            return Err(Error::ParseError("This is not a ping element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in ping element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in ping element."));
        }
        Ok(Ping)
    }
}

impl Into<Element> for Ping {
    fn into(self) -> Element {
        Element::builder("ping")
                .ns(ns::PING)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'/>".parse().unwrap();
        Ping::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'><coucou/></ping>".parse().unwrap();
        let error = Ping::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in ping element.");
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping' coucou=''/>".parse().unwrap();
        let error = Ping::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in ping element.");
    }
}
