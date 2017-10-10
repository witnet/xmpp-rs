// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;

use error::Error;

use ns;

/// Structure representing an `<encryption xmlns='urn:xmpp:eme:0'/>` element.
#[derive(Debug, Clone)]
pub struct ExplicitMessageEncryption {
    /// Namespace of the encryption scheme used.
    pub namespace: String,

    /// User-friendly name for the encryption scheme, should be `None` for OTR,
    /// legacy OpenPGP and OX.
    pub name: Option<String>,
}

impl TryFrom<Element> for ExplicitMessageEncryption {
    type Err = Error;

    fn try_from(elem: Element) -> Result<ExplicitMessageEncryption, Error> {
        check_self!(elem, "encryption", ns::EME);
        check_no_children!(elem, "encryption");
        check_no_unknown_attributes!(elem, "encryption", ["namespace", "name"]);
        Ok(ExplicitMessageEncryption {
            namespace: get_attr!(elem, "namespace", required),
            name: get_attr!(elem, "name", optional),
        })
    }
}

impl From<ExplicitMessageEncryption> for Element {
    fn from(eme: ExplicitMessageEncryption) -> Element {
        Element::builder("encryption")
                .ns(ns::EME)
                .attr("namespace", eme.namespace)
                .attr("name", eme.name)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0' namespace='urn:xmpp:otr:0'/>".parse().unwrap();
        let encryption = ExplicitMessageEncryption::try_from(elem).unwrap();
        assert_eq!(encryption.namespace, "urn:xmpp:otr:0");
        assert_eq!(encryption.name, None);

        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0' namespace='some.unknown.mechanism' name='SuperMechanism'/>".parse().unwrap();
        let encryption = ExplicitMessageEncryption::try_from(elem).unwrap();
        assert_eq!(encryption.namespace, "some.unknown.mechanism");
        assert_eq!(encryption.name, Some(String::from("SuperMechanism")));
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = ExplicitMessageEncryption::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a encryption element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0'><coucou/></encryption>".parse().unwrap();
        let error = ExplicitMessageEncryption::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in encryption element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0' namespace='coucou'/>".parse().unwrap();
        let eme = ExplicitMessageEncryption { namespace: String::from("coucou"), name: None };
        let elem2 = eme.into();
        assert_eq!(elem, elem2);
    }
}
