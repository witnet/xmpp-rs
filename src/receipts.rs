// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(missing_docs)]

generate_empty_element!(
    /// Requests that this message is acked by the final recipient once
    /// received.
    Request, "request", RECEIPTS
);

generate_element!(
    /// Notes that a previous message has correctly been received, it is
    /// referenced by its 'id' attribute.
    Received, "received", RECEIPTS,
    attributes: [
        /// The 'id' attribute of the received message.
        id: Option<String> = "id" => optional,
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use ns;

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
