// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use date::DateTime;

generate_element!(
    History, "history", MUC,
    attributes: [
        maxchars: Option<u32> = "maxchars" => optional,
        maxstanzas: Option<u32> = "maxstanzas" => optional,
        seconds: Option<u32> = "seconds" => optional,
        since: Option<DateTime> = "since" => optional,
    ]
);

generate_element!(
    Muc, "x", MUC, children: [
        password: Option<String> = ("password", MUC) => String,
        history: Option<History> = ("history", MUC) => History
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use error::Error;
    use std::str::FromStr;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_muc_simple() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        Muc::try_from(elem).unwrap();
    }

    #[test]
    fn test_muc_invalid_child() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'><coucou/></x>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_muc_serialise() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        let muc = Muc {
            password: None,
            history: None,
        };
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_muc_invalid_attribute() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc' coucou=''/>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }

    #[test]
    fn test_muc_simple_password() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc'>
                <password>coucou</password>
            </x>"
        .parse().unwrap();
        let elem1 = elem.clone();
        let muc = Muc::try_from(elem).unwrap();
        assert_eq!(muc.password, Some("coucou".to_owned()));

        let elem2 = Element::from(muc);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn history() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc'>
                <history maxstanzas='0'/>
            </x>"
        .parse().unwrap();
        let muc = Muc::try_from(elem).unwrap();
        let history = muc.history.unwrap();
        assert_eq!(history.maxstanzas, Some(0));
        assert_eq!(history.maxchars, None);
        assert_eq!(history.seconds, None);
        assert_eq!(history.since, None);

        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc'>
                <history since='1970-01-01T00:00:00Z'/>
            </x>"
        .parse().unwrap();
        let muc = Muc::try_from(elem).unwrap();
        assert_eq!(muc.history.unwrap().since.unwrap(), DateTime::from_str("1970-01-01T00:00:00+00:00").unwrap());
    }
}
