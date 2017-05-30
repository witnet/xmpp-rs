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
pub struct Muc;

impl TryFrom<Element> for Muc {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Muc, Error> {
        if !elem.is("x", ns::MUC) {
            return Err(Error::ParseError("This is not an x element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in x element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in x element."));
        }
        Ok(Muc)
    }
}

impl Into<Element> for Muc {
    fn into(self) -> Element {
        Element::builder("x")
                .ns(ns::MUC)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        Muc::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'><coucou/></x>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        let muc = Muc;
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc' coucou=''/>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }
}
