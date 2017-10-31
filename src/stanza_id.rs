// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use ns;

generate_element_with_only_attributes!(StanzaId, "stanza-id", ns::SID, [
    id: String = "id" => required,
    by: Jid = "by" => required,
]);

generate_element_with_only_attributes!(OriginId, "origin-id", ns::SID, [
    id: String = "id" => required,
]);

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>".parse().unwrap();
        let stanza_id = StanzaId::try_from(elem).unwrap();
        assert_eq!(stanza_id.id, String::from("coucou"));
        assert_eq!(stanza_id.by, Jid::from_str("coucou@coucou").unwrap());

        let elem: Element = "<origin-id xmlns='urn:xmpp:sid:0' id='coucou'/>".parse().unwrap();
        let origin_id = OriginId::try_from(elem).unwrap();
        assert_eq!(origin_id.id, String::from("coucou"));
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'><coucou/></stanza-id>".parse().unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in stanza-id element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'/>".parse().unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");
    }

    #[test]
    fn test_invalid_by() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou'/>".parse().unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'by' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>".parse().unwrap();
        let stanza_id = StanzaId { id: String::from("coucou"), by: Jid::from_str("coucou@coucou").unwrap() };
        let elem2 = stanza_id.into();
        assert_eq!(elem, elem2);
    }
}
