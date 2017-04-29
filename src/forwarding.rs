// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use delay;
use message;

use ns;

#[derive(Debug, Clone)]
pub struct Forwarded {
    pub delay: Option<delay::Delay>,
    // XXX: really?  Option?
    pub stanza: Option<message::Message>,
}

pub fn parse_forwarded(root: &Element) -> Result<Forwarded, Error> {
    if !root.is("forwarded", ns::FORWARD) {
        return Err(Error::ParseError("This is not a forwarded element."));
    }
    let mut delay = None;
    let mut stanza = None;
    for child in root.children() {
        if child.is("delay", ns::DELAY) {
            delay = Some(delay::parse_delay(child)?);
        } else if child.is("message", ns::JABBER_CLIENT) {
            stanza = Some(message::parse_message(child)?);
        // TODO: also handle the five other possibilities.
        } else {
            return Err(Error::ParseError("Unknown child in forwarded element."));
        }
    }
    Ok(Forwarded {
        delay: delay,
        stanza: stanza,
    })
}

pub fn serialise(forwarded: &Forwarded) -> Element {
    Element::builder("forwarded")
            .ns(ns::FORWARD)
            .append(forwarded.delay.clone())
            .append(forwarded.stanza.clone())
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use forwarding;

    #[test]
    fn test_simple() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        forwarding::parse_forwarded(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'><coucou/></forwarded>".parse().unwrap();
        let error = forwarding::parse_forwarded(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in forwarded element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        let forwarded = forwarding::Forwarded { delay: None, stanza: None };
        let elem2 = forwarding::serialise(&forwarded);
        assert_eq!(elem, elem2);
    }
}
