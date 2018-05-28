// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ibb::Stanza;

generate_id!(StreamId);

generate_element!(Transport, "transport", JINGLE_IBB,
attributes: [
    block_size: u16 = "block-size" => required,
    sid: StreamId = "sid" => required,
    stanza: Stanza = "stanza" => default,
]);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use error::Error;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='3' sid='coucou'/>".parse().unwrap();
        let transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.block_size, 3);
        assert_eq!(transport.sid, StreamId(String::from("coucou")));
        assert_eq!(transport.stanza, Stanza::Iq);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='65536'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "number too large to fit in target type");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='-5'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "invalid digit found in string");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'stanza' attribute.");
    }
}
