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
pub enum Receipt {
    Request,
    Received(String),
}

impl TryFrom<Element> for Receipt {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Receipt, Error> {
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in receipt element."));
        }
        if elem.is("request", ns::RECEIPTS) {
            Ok(Receipt::Request)
        } else if elem.is("received", ns::RECEIPTS) {
            let id = elem.attr("id").unwrap_or("").to_owned();
            Ok(Receipt::Received(id))
        } else {
            Err(Error::ParseError("This is not a receipt element."))
        }
    }
}

impl Into<Element> for Receipt {
    fn into(self) -> Element {
        match self {
            Receipt::Request => Element::builder("request")
                                        .ns(ns::RECEIPTS)
                                        .build(),
            Receipt::Received(id) => Element::builder("received")
                                             .ns(ns::RECEIPTS)
                                             .attr("id", id)
                                             .build(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<request xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        Receipt::try_from(elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        Receipt::try_from(elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts' id='coucou'/>".parse().unwrap();
        Receipt::try_from(elem).unwrap();
    }

    #[test]
    fn test_serialise() {
        let receipt = Receipt::Request;
        let elem: Element = receipt.into();
        assert!(elem.is("request", ns::RECEIPTS));

        let receipt = Receipt::Received("coucou".to_owned());
        let elem: Element = receipt.into();
        assert!(elem.is("received", ns::RECEIPTS));
        assert_eq!(elem.attr("id"), Some("coucou"));
    }
}
