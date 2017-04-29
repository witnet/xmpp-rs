// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Ping;

pub fn parse_ping(root: &Element) -> Result<Ping, Error> {
    if !root.is("ping", ns::PING) {
        return Err(Error::ParseError("This is not a ping element."));
    }

    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in ping element."));
    }
    Ok(Ping {  })
}

pub fn serialise_ping() -> Element {
    Element::builder("ping").ns(ns::PING).build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use ping;

    #[test]
    fn test_simple() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'/>".parse().unwrap();
        ping::parse_ping(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'><coucou/></ping>".parse().unwrap();
        let error = ping::parse_ping(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in ping element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping' coucou=''/>".parse().unwrap();
        let error = ping::parse_ping(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in ping element.");
    }
}
