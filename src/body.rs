// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

pub type Body = String;

pub fn parse_body(root: &Element) -> Result<Body, Error> {
    // TODO: also support components and servers.
    if !root.is("body", ns::JABBER_CLIENT) {
        return Err(Error::ParseError("This is not a body element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in body element."));
    }
    Ok(root.text())
}

pub fn serialise(body: &Body) -> Element {
    Element::builder("body")
            .ns(ns::JABBER_CLIENT)
            .append(body.to_owned())
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use body;
    use ns;

    #[test]
    fn test_simple() {
        let elem: Element = "<body xmlns='jabber:client'/>".parse().unwrap();
        body::parse_body(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<body xmlns='jabber:server'/>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a body element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<body xmlns='jabber:client'><coucou/></body>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in body element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<body xmlns='jabber:client' coucou=''/>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in body element.");
    }

    #[test]
    fn test_serialise() {
        let body = body::Body::from("Hello world!");
        let elem = body::serialise(&body);
        assert!(elem.is("body", ns::JABBER_CLIENT));
    }
}
