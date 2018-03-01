// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use error::Error;
use helpers::Base64;
use ns;

generate_element_with_text!(Handshake, "handshake", ns::COMPONENT,
    data: Base64<Vec<u8>>
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<handshake xmlns='jabber:component:accept'/>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert!(handshake.data.is_empty());

        let elem: Element = "<handshake xmlns='jabber:component:accept'>AAAA</handshake>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert_eq!(handshake.data, b"\0\0\0");
    }
}
