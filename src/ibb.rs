// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};
use base64;

use error::Error;

use ns;

generate_attribute!(Stanza, "stanza", {
    Iq => "iq",
    Message => "message",
}, Default = Iq);

#[derive(Debug, Clone)]
pub struct Open {
    pub block_size: u16,
    pub sid: String,
    pub stanza: Stanza,
}

impl TryFrom<Element> for Open {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Open, Error> {
        if !elem.is("open", ns::IBB) {
            return Err(Error::ParseError("This is not an open element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in open element."));
        }
        Ok(Open {
            block_size: get_attr!(elem, "block-size", required),
            sid: get_attr!(elem, "sid", required),
            stanza: get_attr!(elem, "stanza", default),
        })
    }
}

impl From<Open> for Element {
    fn from(open: Open) -> Element {
        Element::builder("open")
                .ns(ns::IBB)
                .attr("block-size", open.block_size)
                .attr("sid", open.sid)
                .attr("stanza", open.stanza)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub seq: u16,
    pub sid: String,
    pub data: Vec<u8>,
}

impl TryFrom<Element> for Data {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Data, Error> {
        if !elem.is("data", ns::IBB) {
            return Err(Error::ParseError("This is not a data element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in data element."));
        }
        Ok(Data {
            seq: get_attr!(elem, "seq", required),
            sid: get_attr!(elem, "sid", required),
            data: base64::decode(&elem.text())?,
        })
    }
}

impl From<Data> for Element {
    fn from(data: Data) -> Element {
        Element::builder("data")
                .ns(ns::IBB)
                .attr("seq", data.seq)
                .attr("sid", data.sid)
                .append(base64::encode(&data.data))
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Close {
    pub sid: String,
}

impl TryFrom<Element> for Close {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Close, Error> {
        if !elem.is("close", ns::IBB) {
            return Err(Error::ParseError("This is not a close element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in close element."));
        }
        Ok(Close {
            sid: get_attr!(elem, "sid", required),
        })
    }
}

impl From<Close> for Element {
    fn from(close: Close) -> Element {
        Element::builder("close")
                .ns(ns::IBB)
                .attr("sid", close.sid)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='3' sid='coucou'/>".parse().unwrap();
        let open = Open::try_from(elem).unwrap();
        assert_eq!(open.block_size, 3);
        assert_eq!(open.sid, "coucou");
        assert_eq!(open.stanza, Stanza::Iq);

        let elem: Element = "<data xmlns='http://jabber.org/protocol/ibb' seq='0' sid='coucou'>AAAA</data>".parse().unwrap();
        let data = Data::try_from(elem).unwrap();
        assert_eq!(data.seq, 0);
        assert_eq!(data.sid, "coucou");
        assert_eq!(data.data, vec!(0, 0, 0));

        let elem: Element = "<close xmlns='http://jabber.org/protocol/ibb' sid='coucou'/>".parse().unwrap();
        let close = Close::try_from(elem).unwrap();
        assert_eq!(close.sid, "coucou");
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='-5'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "invalid digit found in string");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'stanza' attribute.");
    }
}
