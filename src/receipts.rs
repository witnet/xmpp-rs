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
pub enum Receipt {
    Request,
    Received(Option<String>),
}

impl TryFrom<Element> for Receipt {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Receipt, Error> {
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in receipt element."));
        }
        if elem.is("request", ns::RECEIPTS) {
            for _ in elem.attrs() {
                return Err(Error::ParseError("Unknown attribute in request element."));
            }
            Ok(Receipt::Request)
        } else if elem.is("received", ns::RECEIPTS) {
            for (attr, _) in elem.attrs() {
                if attr != "id" {
                    return Err(Error::ParseError("Unknown attribute in received element."));
                }
            }
            let id = get_attr!(elem, "id", optional);
            Ok(Receipt::Received(id))
        } else {
            Err(Error::ParseError("This is not a receipt element."))
        }
    }
}

impl From<Receipt> for Element {
    fn from(receipt: Receipt) -> Element {
        match receipt {
            Receipt::Request => Element::builder("request")
                                        .ns(ns::RECEIPTS),
            Receipt::Received(id) => Element::builder("received")
                                             .ns(ns::RECEIPTS)
                                             .attr("id", id),
        }.build()
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

        let receipt = Receipt::Received(Some(String::from("coucou")));
        let elem: Element = receipt.into();
        assert!(elem.is("received", ns::RECEIPTS));
        assert_eq!(elem.attr("id"), Some("coucou"));
    }
}
