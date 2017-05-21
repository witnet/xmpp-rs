// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

use ibb::Stanza;

#[derive(Debug, Clone)]
pub struct Transport {
    pub block_size: u16,
    pub sid: String,
    pub stanza: Stanza,
}

impl<'a> TryFrom<&'a Element> for Transport {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Transport, Error> {
        if elem.is("transport", ns::JINGLE_IBB) {
            for _ in elem.children() {
                return Err(Error::ParseError("Unknown child in JingleIBB element."));
            }
            let block_size = get_attr!(elem, "block-size", required);
            let sid = get_attr!(elem, "sid", required);
            let stanza = get_attr!(elem, "stanza", default);
            Ok(Transport {
                block_size: block_size,
                sid: sid,
                stanza: stanza
            })
        } else {
            Err(Error::ParseError("This is not an JingleIBB element."))
        }
    }
}

impl<'a> Into<Element> for &'a Transport {
    fn into(self) -> Element {
        Element::builder("transport")
                .ns(ns::JINGLE_IBB)
                .attr("block-size", format!("{}", self.block_size))
                .attr("sid", self.sid.clone())
                .attr("stanza", self.stanza.clone())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='3' sid='coucou'/>".parse().unwrap();
        let transport = Transport::try_from(&elem).unwrap();
        assert_eq!(transport.block_size, 3);
        assert_eq!(transport.sid, "coucou");
        assert_eq!(transport.stanza, Stanza::Iq);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1'/>".parse().unwrap();
        let error = Transport::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='65536'/>".parse().unwrap();
        let error = Transport::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "number too large to fit in target type");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='-5'/>".parse().unwrap();
        let error = Transport::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "invalid digit found in string");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128'/>".parse().unwrap();
        let error = Transport::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = Transport::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid 'stanza' attribute.");
    }
}
