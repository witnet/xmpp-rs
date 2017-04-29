// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub enum Receipt {
    Request,
    Received(String),
}

pub fn parse_receipt(root: &Element) -> Result<Receipt, Error> {
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in receipt element."));
    }
    if root.is("request", ns::RECEIPTS) {
        Ok(Receipt::Request)
    } else if root.is("received", ns::RECEIPTS) {
        let id = root.attr("id").unwrap_or("").to_owned();
        Ok(Receipt::Received(id))
    } else {
        Err(Error::ParseError("This is not a receipt element."))
    }
}

pub fn serialise(receipt: &Receipt) -> Element {
    match *receipt {
        Receipt::Request => Element::builder("request")
                                    .ns(ns::RECEIPTS)
                                    .build(),
        Receipt::Received(ref id) => Element::builder("received")
                                             .ns(ns::RECEIPTS)
                                             .attr("id", id.clone())
                                             .build(),
    }
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    //use error::Error;
    use receipts;
    use ns;

    #[test]
    fn test_simple() {
        let elem: Element = "<request xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts' id='coucou'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();
    }

    #[test]
    fn test_serialise() {
        let receipt = receipts::Receipt::Request;
        let elem = receipts::serialise(&receipt);
        assert!(elem.is("request", ns::RECEIPTS));

        let receipt = receipts::Receipt::Received("coucou".to_owned());
        let elem = receipts::serialise(&receipt);
        assert!(elem.is("received", ns::RECEIPTS));
        assert_eq!(elem.attr("id"), Some("coucou"));
    }
}
