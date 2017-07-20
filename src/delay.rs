// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use chrono::{DateTime, FixedOffset};

use error::Error;
use jid::Jid;

use ns;

#[derive(Debug, Clone)]
pub struct Delay {
    pub from: Option<Jid>,
    pub stamp: DateTime<FixedOffset>,
    pub data: Option<String>,
}

impl TryFrom<Element> for Delay {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Delay, Error> {
        if !elem.is("delay", ns::DELAY) {
            return Err(Error::ParseError("This is not a delay element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in delay element."));
        }
        let from = get_attr!(elem, "from", optional);
        let stamp = get_attr!(elem, "stamp", required, stamp, DateTime::parse_from_rfc3339(stamp)?);
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

impl Into<Element> for Delay {
    fn into(self) -> Element {
        Element::builder("delay")
                .ns(ns::DELAY)
                .attr("from", self.from.and_then(|value| Some(String::from(value))))
                .attr("stamp", self.stamp.to_rfc3339())
                .append(self.data)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_simple() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = Delay::try_from(elem).unwrap();
        assert_eq!(delay.from, Some(Jid::from_str("capulet.com").unwrap()));
        assert_eq!(delay.stamp.year(), 2002);
        assert_eq!(delay.stamp.month(), 9);
        assert_eq!(delay.stamp.day(), 10);
        assert_eq!(delay.stamp.hour(), 23);
        assert_eq!(delay.stamp.minute(), 08);
        assert_eq!(delay.stamp.second(), 25);
        assert_eq!(delay.stamp.nanosecond(), 0);
        assert_eq!(delay.stamp.timezone(), FixedOffset::east(0));
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
            stamp: DateTime::parse_from_rfc3339("2002-09-10T23:08:25Z").unwrap(),
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
            stamp: DateTime::parse_from_rfc3339("2002-09-10T23:08:25Z").unwrap(),
            data: Some(String::from("Reason")),
        };
        let elem2 = delay.into();
        assert_eq!(elem, elem2);
    }
}
