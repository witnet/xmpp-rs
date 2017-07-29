// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;

use error::Error;

use delay::Delay;
use message::Message;

use ns;

#[derive(Debug, Clone)]
pub struct Forwarded {
    pub delay: Option<Delay>,
    // XXX: really?  Option?
    pub stanza: Option<Message>,
}

impl TryFrom<Element> for Forwarded {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Forwarded, Error> {
        if !elem.is("forwarded", ns::FORWARD) {
            return Err(Error::ParseError("This is not a forwarded element."));
        }
        let mut delay = None;
        let mut stanza = None;
        for child in elem.children() {
            if child.is("delay", ns::DELAY) {
                delay = Some(Delay::try_from(child.clone())?);
            } else if child.is("message", ns::DEFAULT_NS) {
                stanza = Some(Message::try_from(child.clone())?);
            // TODO: also handle the two other possibilities.
            } else {
                return Err(Error::ParseError("Unknown child in forwarded element."));
            }
        }
        Ok(Forwarded {
            delay: delay,
            stanza: stanza,
        })
    }
}

impl From<Forwarded> for Element {
    fn from(forwarded: Forwarded) -> Element {
        Element::builder("forwarded")
                .ns(ns::FORWARD)
                .append(forwarded.delay.map(Element::from))
                .append(forwarded.stanza.map(Element::from))
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        Forwarded::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'><coucou/></forwarded>".parse().unwrap();
        let error = Forwarded::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in forwarded element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        let forwarded = Forwarded { delay: None, stanza: None };
        let elem2 = forwarded.into();
        assert_eq!(elem, elem2);
    }
}
