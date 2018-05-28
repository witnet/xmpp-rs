// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use iq::{IqGetPayload, IqResultPayload};

generate_element!(
    Version, "query", VERSION,
    children: [
        name: Required<String> = ("name", VERSION) => String,
        version: Required<String> = ("version", VERSION) => String,
        os: Option<String> = ("os", VERSION) => String
    ]
);

impl IqGetPayload for Version {}
impl IqResultPayload for Version {}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='jabber:iq:version'><name>xmpp-rs</name><version>0.3.0</version></query>".parse().unwrap();
        let version = Version::try_from(elem).unwrap();
        assert_eq!(version.name, String::from("xmpp-rs"));
        assert_eq!(version.version, String::from("0.3.0"));
        assert_eq!(version.os, None);
    }

    #[test]
    fn serialisation() {
        let version = Version {
            name: String::from("xmpp-rs"),
            version: String::from("0.3.0"),
            os: None,
        };
        let elem1 = Element::from(version);
        let elem2: Element = "<query xmlns='jabber:iq:version'><name>xmpp-rs</name><version>0.3.0</version></query>".parse().unwrap();
        println!("{:?}", elem1);
        assert!(elem1.compare_to(&elem2));
    }
}
