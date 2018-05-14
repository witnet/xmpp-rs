// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ns;
use helpers::Base64;

generate_attribute!(Mechanism, "mechanism", {
    Plain => "PLAIN",
    ScramSha1 => "SCRAM-SHA-1",
    Anonymous => "ANONYMOUS",
});

generate_element_with_text!(Auth, "auth", ns::SASL,
    [
        mechanism: Mechanism = "mechanism" => required
    ],
    data: Base64<Vec<u8>>
);

generate_element_with_text!(Challenge, "challenge", ns::SASL,
    data: Base64<Vec<u8>>
);

generate_element_with_text!(Response, "response", ns::SASL,
    data: Base64<Vec<u8>>
);

generate_element_with_text!(Success, "success", ns::SASL,
    data: Base64<Vec<u8>>
);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;

    #[test]
    fn test_simple() {
        let elem: Element = "<auth xmlns='urn:ietf:params:xml:ns:xmpp-sasl' mechanism='PLAIN'/>".parse().unwrap();
        let auth = Auth::try_from(elem).unwrap();
        assert_eq!(auth.mechanism, Mechanism::Plain);
        assert!(auth.data.is_empty());
    }
}
