// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;
use jid::Jid;

use ns;

#[derive(Debug, Clone)]
pub struct Delay {
    pub from: Option<Jid>,
    pub stamp: String,
    pub data: Option<String>,
}

impl<'a> TryFrom<&'a Element> for Delay {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Delay, Error> {
        if !elem.is("delay", ns::DELAY) {
            return Err(Error::ParseError("This is not a delay element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in delay element."));
        }
        let from = elem.attr("from").and_then(|value| value.parse().ok());
        let stamp = elem.attr("stamp").ok_or(Error::ParseError("Mandatory argument 'stamp' not present in delay element."))?.to_owned();
        let data = match elem.text().as_ref() {
            "" => None,
            text => Some(text.to_owned()),
        };
        Ok(Delay {
            from: from,
            stamp: stamp,
            data: data,
        })
    }
}

impl<'a> Into<Element> for &'a Delay {
    fn into(self) -> Element {
        Element::builder("delay")
                .ns(ns::DELAY)
                .attr("from", self.from.clone().and_then(|value| Some(String::from(value))))
                .attr("stamp", self.stamp.clone())
                .append(self.data.clone())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = Delay::try_from(&elem).unwrap();
        assert_eq!(delay.from, Some(Jid::from_str("capulet.com").unwrap()));
        assert_eq!(delay.stamp, "2002-09-10T23:08:25Z");
        assert_eq!(delay.data, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Delay::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a delay element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay'><coucou/></delay>".parse().unwrap();
        let error = Delay::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in delay element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = Delay {
            from: None,
            stamp: "2002-09-10T23:08:25Z".to_owned(),
            data: None,
        };
        let elem2 = (&delay).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialise_data() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='juliet@example.org' stamp='2002-09-10T23:08:25Z'>Reason</delay>".parse().unwrap();
        let delay = Delay {
            from: Some(Jid::from_str("juliet@example.org").unwrap()),
            stamp: "2002-09-10T23:08:25Z".to_owned(),
            data: Some(String::from("Reason")),
        };
        let elem2 = (&delay).into();
        assert_eq!(elem, elem2);
    }
}
