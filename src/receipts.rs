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
pub struct Request;

impl TryFrom<Element> for Request {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Request, Error> {
        if !elem.is("request", ns::RECEIPTS) {
            return Err(Error::ParseError("This is not a request element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in request element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in request element."));
        }
        Ok(Request)
    }
}

impl From<Request> for Element {
    fn from(_: Request) -> Element {
        Element::builder("request")
                .ns(ns::RECEIPTS)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Received {
    pub id: Option<String>,
}

impl TryFrom<Element> for Received {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Received, Error> {
        if !elem.is("received", ns::RECEIPTS) {
            return Err(Error::ParseError("This is not a received element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in received element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "id" {
                return Err(Error::ParseError("Unknown attribute in received element."));
            }
        }
        Ok(Received {
            id: get_attr!(elem, "id", optional),
        })
    }
}

impl From<Received> for Element {
    fn from(received: Received) -> Element {
        Element::builder("received")
               .ns(ns::RECEIPTS)
               .attr("id", received.id)
               .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<request xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        Request::try_from(elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        Received::try_from(elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts' id='coucou'/>".parse().unwrap();
        Received::try_from(elem).unwrap();
    }

    #[test]
    fn test_serialise() {
        let receipt = Request;
        let elem: Element = receipt.into();
        assert!(elem.is("request", ns::RECEIPTS));
        assert_eq!(elem.attrs().count(), 0);

        let receipt = Received {
            id: Some(String::from("coucou")),
        };
        let elem: Element = receipt.into();
        assert!(elem.is("received", ns::RECEIPTS));
        assert_eq!(elem.attr("id"), Some("coucou"));
    }
}
