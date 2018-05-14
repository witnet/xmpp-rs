// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use error::Error;
use helpers::PlainText;
use ns;

use sha1::Sha1;
use digest::Digest;

generate_element_with_text!(Handshake, "handshake", ns::COMPONENT,
    data: PlainText<Option<String>>
);

impl Handshake {
    pub fn new() -> Handshake {
        Handshake {
            data: None,
        }
    }

    pub fn from_password_and_stream_id(password: &str, stream_id: &str) -> Handshake {
        let input = String::from(stream_id) + password;
        let hash = Sha1::digest(input.as_bytes());
        let content = format!("{:x}", hash);
        Handshake {
            data: Some(content),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;

    #[test]
    fn test_simple() {
        let elem: Element = "<handshake xmlns='jabber:component:accept'/>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert_eq!(handshake.data, None);

        let elem: Element = "<handshake xmlns='jabber:component:accept'>Coucou</handshake>".parse().unwrap();
        let handshake = Handshake::try_from(elem).unwrap();
        assert_eq!(handshake.data, Some(String::from("Coucou")));
    }

    #[test]
    fn test_constructors() {
        let handshake = Handshake::new();
        assert_eq!(handshake.data, None);

        let handshake = Handshake::from_password_and_stream_id("123456", "sid");
        assert_eq!(handshake.data, Some(String::from("9accec263ab84a43c6037ccf7cd48cb1d3f6df8e")));
    }
}
