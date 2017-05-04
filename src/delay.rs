// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::{Element, IntoElements, ElementEmitter};

use error::Error;
use jid::Jid;

use ns;

#[derive(Debug, Clone)]
pub struct Delay {
    pub from: Option<Jid>,
    pub stamp: String,
    pub data: Option<String>,
}

pub fn parse_delay(root: &Element) -> Result<Delay, Error> {
    if !root.is("delay", ns::DELAY) {
        return Err(Error::ParseError("This is not a delay element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in delay element."));
    }
    let from = root.attr("from").and_then(|value| value.parse().ok());
    let stamp = root.attr("stamp").ok_or(Error::ParseError("Mandatory argument 'stamp' not present in delay element."))?.to_owned();
    let data = match root.text().as_ref() {
        "" => None,
        text => Some(text.to_owned()),
    };
    Ok(Delay {
        from: from,
        stamp: stamp,
        data: data,
    })
}

pub fn serialise(delay: &Delay) -> Element {
    Element::builder("delay")
            .ns(ns::DELAY)
            .attr("from", delay.from.clone().and_then(|value| Some(String::from(value))))
            .attr("stamp", delay.stamp.clone())
            .append(delay.data.clone())
            .build()
}

impl IntoElements for Delay {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let elem = serialise(&self);
        emitter.append_child(elem)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use minidom::Element;
    use error::Error;
    use jid::Jid;
    use delay;

    #[test]
    fn test_simple() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = delay::parse_delay(&elem).unwrap();
        assert_eq!(delay.from, Some(Jid::from_str("capulet.com").unwrap()));
        assert_eq!(delay.stamp, "2002-09-10T23:08:25Z");
        assert_eq!(delay.data, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = delay::parse_delay(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a delay element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay'><coucou/></delay>".parse().unwrap();
        let error = delay::parse_delay(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in delay element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = delay::Delay {
            from: None,
            stamp: "2002-09-10T23:08:25Z".to_owned(),
            data: None,
        };
        let elem2 = delay::serialise(&delay);
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialise_data() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='juliet@example.org' stamp='2002-09-10T23:08:25Z'>Reason</delay>".parse().unwrap();
        let delay = delay::Delay {
            from: Some(Jid::from_str("juliet@example.org").unwrap()),
            stamp: "2002-09-10T23:08:25Z".to_owned(),
            data: Some(String::from("Reason")),
        };
        let elem2 = delay::serialise(&delay);
        assert_eq!(elem, elem2);
    }
}
