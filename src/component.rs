// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use error::Error;
use helpers::PlainText;
use ns;

generate_element_with_text!(Handshake, "handshake", ns::COMPONENT,
    data: PlainText<Option<String>>
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<handshake xmlns='jabber:component:accept'/>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert_eq!(handshake.data, None);

        let elem: Element = "<handshake xmlns='jabber:component:accept'>Coucou</handshake>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert_eq!(handshake.data, Some(String::from("Coucou")));
    }
}
