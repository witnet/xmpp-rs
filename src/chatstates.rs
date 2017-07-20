// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;

use error::Error;

use ns;

/// Enum representing chatstate elements part of the
/// `http://jabber.org/protocol/chatstates` namespace.
#[derive(Debug, Clone)]
pub enum ChatState {
    /// `<active xmlns='http://jabber.org/protocol/chatstates'/>`
    Active,

    /// `<composing xmlns='http://jabber.org/protocol/chatstates'/>`
    Composing,

    /// `<gone xmlns='http://jabber.org/protocol/chatstates'/>`
    Gone,

    /// `<inactive xmlns='http://jabber.org/protocol/chatstates'/>`
    Inactive,

    /// `<paused xmlns='http://jabber.org/protocol/chatstates'/>`
    Paused,
}

impl TryFrom<Element> for ChatState {
    type Err = Error;

    fn try_from(elem: Element) -> Result<ChatState, Error> {
        if elem.ns() != Some(ns::CHATSTATES) {
            return Err(Error::ParseError("This is not a chatstate element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in chatstate element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in chatstate element."));
        }
        Ok(match elem.name() {
            "active" => ChatState::Active,
            "composing" => ChatState::Composing,
            "gone" => ChatState::Gone,
            "inactive" => ChatState::Inactive,
            "paused" => ChatState::Paused,
            _ => return Err(Error::ParseError("This is not a chatstate element.")),
        })
    }
}

impl From<ChatState> for Element {
    fn from(chatstate: ChatState) -> Element {
        Element::builder(match chatstate {
            ChatState::Active => "active",
            ChatState::Composing => "composing",
            ChatState::Gone => "gone",
            ChatState::Inactive => "inactive",
            ChatState::Paused => "paused",
        }).ns(ns::CHATSTATES)
          .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<active xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        ChatState::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<coucou xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a chatstate element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<gone xmlns='http://jabber.org/protocol/chatstates'><coucou/></gone>".parse().unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in chatstate element.");
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<inactive xmlns='http://jabber.org/protocol/chatstates' coucou=''/>".parse().unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in chatstate element.");
    }

    #[test]
    fn test_serialise() {
        let chatstate = ChatState::Active;
        let elem: Element = chatstate.into();
        assert!(elem.is("active", ns::CHATSTATES));
    }
}
