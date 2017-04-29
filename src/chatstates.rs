// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub enum ChatState {
    Active,
    Composing,
    Gone,
    Inactive,
    Paused,
}

pub fn parse_chatstate(root: &Element) -> Result<ChatState, Error> {
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in chatstate element."));
    }
    if root.is("active", ns::CHATSTATES) {
        Ok(ChatState::Active)
    } else if root.is("composing", ns::CHATSTATES) {
        Ok(ChatState::Composing)
    } else if root.is("gone", ns::CHATSTATES) {
        Ok(ChatState::Gone)
    } else if root.is("inactive", ns::CHATSTATES) {
        Ok(ChatState::Inactive)
    } else if root.is("paused", ns::CHATSTATES) {
        Ok(ChatState::Paused)
    } else {
        Err(Error::ParseError("This is not a chatstate element."))
    }
}

pub fn serialise(chatstate: &ChatState) -> Element {
    Element::builder(match *chatstate {
        ChatState::Active => "active",
        ChatState::Composing => "composing",
        ChatState::Gone => "gone",
        ChatState::Inactive => "inactive",
        ChatState::Paused => "paused",
    }).ns(ns::CHATSTATES)
      .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use chatstates;
    use ns;

    #[test]
    fn test_simple() {
        let elem: Element = "<active xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        chatstates::parse_chatstate(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<coucou xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a chatstate element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<gone xmlns='http://jabber.org/protocol/chatstates'><coucou/></gone>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in chatstate element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<inactive xmlns='http://jabber.org/protocol/chatstates' coucou=''/>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in chatstate element.");
    }

    #[test]
    fn test_serialise() {
        let chatstate = chatstates::ChatState::Active;
        let elem = chatstates::serialise(&chatstate);
        assert!(elem.is("active", ns::CHATSTATES));
    }
}
