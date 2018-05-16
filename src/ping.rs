// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use iq::IqGetPayload;

generate_empty_element!(Ping, "ping", PING);

impl IqGetPayload for Ping {}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use error::Error;

    #[test]
    fn test_simple() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'/>".parse().unwrap();
        Ping::try_from(elem).unwrap();
    }

    #[test]
    fn test_serialise() {
        let elem1 = Element::from(Ping);
        let elem2: Element = "<ping xmlns='urn:xmpp:ping'/>".parse().unwrap();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'><coucou/></ping>".parse().unwrap();
        let error = Ping::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in ping element.");
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping' coucou=''/>".parse().unwrap();
        let error = Ping::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in ping element.");
    }
}
