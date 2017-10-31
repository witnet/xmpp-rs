// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use date::DateTime;

use error::Error;
use jid::Jid;

use ns;

#[derive(Debug, Clone)]
pub struct Delay {
    pub from: Option<Jid>,
    pub stamp: DateTime,
    pub data: Option<String>,
}

impl TryFrom<Element> for Delay {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Delay, Error> {
        check_self!(elem, "delay", ns::DELAY);
        check_no_children!(elem, "delay");
        check_no_unknown_attributes!(elem, "delay", ["from", "stamp"]);
        let data = match elem.text().as_ref() {
            "" => None,
            text => Some(text.to_owned()),
        };
        Ok(Delay {
            from: get_attr!(elem, "from", optional),
            stamp: get_attr!(elem, "stamp", required),
            data: data,
        })
    }
}

impl From<Delay> for Element {
    fn from(delay: Delay) -> Element {
        Element::builder("delay")
                .ns(ns::DELAY)
                .attr("from", delay.from)
                .attr("stamp", delay.stamp)
                .append(delay.data)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = Delay::try_from(elem).unwrap();
        assert_eq!(delay.from, Some(Jid::from_str("capulet.com").unwrap()));
        assert_eq!(delay.stamp, DateTime::from_str("2002-09-10T23:08:25Z").unwrap());
        assert_eq!(delay.data, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Delay::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a delay element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay'><coucou/></delay>".parse().unwrap();
        let error = Delay::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in delay element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' stamp='2002-09-10T23:08:25+00:00'/>".parse().unwrap();
        let delay = Delay {
            from: None,
            stamp: DateTime::from_str("2002-09-10T23:08:25Z").unwrap(),
            data: None,
        };
        let elem2 = delay.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialise_data() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='juliet@example.org' stamp='2002-09-10T23:08:25+00:00'>Reason</delay>".parse().unwrap();
        let delay = Delay {
            from: Some(Jid::from_str("juliet@example.org").unwrap()),
            stamp: DateTime::from_str("2002-09-10T23:08:25Z").unwrap(),
            data: Some(String::from("Reason")),
        };
        let elem2 = delay.into();
        assert_eq!(elem, elem2);
    }
}
