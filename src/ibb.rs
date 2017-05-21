// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};
use base64;

use error::Error;

use ns;

#[derive(Debug, Clone, PartialEq)]
pub enum Stanza {
    Iq,
    Message,
}

impl Default for Stanza {
    fn default() -> Stanza {
        Stanza::Iq
    }
}

impl FromStr for Stanza {
    type Err = Error;

    fn from_str(s: &str) -> Result<Stanza, Error> {
        Ok(match s {
            "iq" => Stanza::Iq,
            "message" => Stanza::Message,

            _ => return Err(Error::ParseError("Invalid 'stanza' attribute.")),
        })
    }
}

impl IntoAttributeValue for Stanza {
    fn into_attribute_value(self) -> Option<String> {
        match self {
            Stanza::Iq => None,
            Stanza::Message => Some(String::from("message")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IBB {
    Open {
        block_size: u16,
        sid: String,
        stanza: Stanza,
    },
    Data {
        seq: u16,
        sid: String,
        data: Vec<u8>,
    },
    Close {
        sid: String,
    },
}

impl<'a> TryFrom<&'a Element> for IBB {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<IBB, Error> {
        if elem.is("open", ns::IBB) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in open element."));
            }
            let block_size = get_attr!(elem, "block-size", required, block_size, block_size.parse()?);
            let sid = get_attr!(elem, "sid", required, sid, sid.parse()?);
            let stanza = get_attr!(elem, "stanza", default, stanza, stanza.parse()?);
            Ok(IBB::Open {
                block_size: block_size,
                sid: sid,
                stanza: stanza
            })
        } else if elem.is("data", ns::IBB) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in data element."));
            }
            let seq = get_attr!(elem, "seq", required, seq, seq.parse()?);
            let sid = get_attr!(elem, "sid", required, sid, sid.parse()?);
            let data = base64::decode(&elem.text())?;
            Ok(IBB::Data {
                seq: seq,
                sid: sid,
                data: data
            })
        } else if elem.is("close", ns::IBB) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in close element."));
            }
            let sid = get_attr!(elem, "sid", required, sid, sid.parse()?);
            Ok(IBB::Close {
                sid: sid,
            })
        } else {
            Err(Error::ParseError("This is not an ibb element."))
        }
    }
}

impl<'a> Into<Element> for &'a IBB {
    fn into(self) -> Element {
        match *self {
            IBB::Open { ref block_size, ref sid, ref stanza } => {
                Element::builder("open")
                        .ns(ns::IBB)
                        .attr("block-size", format!("{}", block_size))
                        .attr("sid", sid.to_owned())
                        .attr("stanza", stanza.to_owned())
                        .build()
            },
            IBB::Data { ref seq, ref sid, ref data } => {
                Element::builder("data")
                        .ns(ns::IBB)
                        .attr("seq", format!("{}", seq))
                        .attr("sid", sid.to_owned())
                        .append(base64::encode(&data))
                        .build()
            },
            IBB::Close { ref sid } => {
                Element::builder("close")
                        .ns(ns::IBB)
                        .attr("sid", sid.to_owned())
                        .build()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='3' sid='coucou'/>".parse().unwrap();
        let open = IBB::try_from(&elem).unwrap();
        match open {
            IBB::Open { block_size, sid, stanza } => {
                assert_eq!(block_size, 3);
                assert_eq!(sid, "coucou");
                assert_eq!(stanza, Stanza::Iq);
            },
            _ => panic!(),
        }

        let elem: Element = "<data xmlns='http://jabber.org/protocol/ibb' seq='0' sid='coucou'>AAAA</data>".parse().unwrap();
        let data = IBB::try_from(&elem).unwrap();
        match data {
            IBB::Data { seq, sid, data } => {
                assert_eq!(seq, 0);
                assert_eq!(sid, "coucou");
                assert_eq!(data, vec!(0, 0, 0));
            },
            _ => panic!(),
        }

        let elem: Element = "<close xmlns='http://jabber.org/protocol/ibb' sid='coucou'/>".parse().unwrap();
        let close = IBB::try_from(&elem).unwrap();
        match close {
            IBB::Close { sid } => {
                assert_eq!(sid, "coucou");
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb'/>".parse().unwrap();
        let error = IBB::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='-5'/>".parse().unwrap();
        let error = IBB::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "invalid digit found in string");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128'/>".parse().unwrap();
        let error = IBB::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = IBB::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid 'stanza' attribute.");
    }
}
