// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

pub type Status = String;

pub fn parse_status(root: &Element) -> Result<Status, Error> {
    // TODO: also support components and servers.
    if !root.is("status", ns::JABBER_CLIENT) {
        return Err(Error::ParseError("This is not a status element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in status element."));
    }
    Ok(root.text())
}

pub fn serialise(status: &Status) -> Element {
    Element::builder("status")
            .ns(ns::JABBER_CLIENT)
            .append(status.to_owned())
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use status;
    use ns;

    #[test]
    fn test_simple() {
        let elem: Element = "<status xmlns='jabber:client'/>".parse().unwrap();
        status::parse_status(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<status xmlns='jabber:server'/>".parse().unwrap();
        let error = status::parse_status(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a status element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<status xmlns='jabber:client'><coucou/></status>".parse().unwrap();
        let error = status::parse_status(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<status xmlns='jabber:client' coucou=''/>".parse().unwrap();
        let error = status::parse_status(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in status element.");
    }

    #[test]
    fn test_serialise() {
        let status = status::Status::from("Hello world!");
        let elem = status::serialise(&status);
        assert!(elem.is("status", ns::JABBER_CLIENT));
    }
}
