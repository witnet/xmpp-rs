// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;
use jid::Jid;
use error::Error;
use ns;

generate_element_with_only_attributes!(Open, "open", ns::WEBSOCKET, [
    from: Option<Jid> = "from" => optional,
    to: Option<Jid> = "to" => optional,
    id: Option<String> = "id" => optional,
    version: Option<String> = "version" => optional,
    xml_lang: Option<String> = "xml:lang" => optional,
]);

impl Open {
    pub fn new(to: Jid) -> Open {
        Open {
            from: None,
            to: Some(to),
            id: None,
            version: Some(String::from("1.0")),
            xml_lang: None,
        }
    }

    pub fn with_from(mut self, from: Jid) -> Open {
        self.from = Some(from);
        self
    }

    pub fn with_id(mut self, id: String) -> Open {
        self.id = Some(id);
        self
    }

    pub fn with_lang(mut self, xml_lang: String) -> Open {
        self.xml_lang = Some(xml_lang);
        self
    }

    pub fn is_version(&self, version: &str) -> bool {
        match self.version {
            None => false,
            Some(ref self_version) => self_version == &String::from(version),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;

    #[test]
    fn test_simple() {
        let elem: Element = "<open xmlns='urn:ietf:params:xml:ns:xmpp-framing'/>".parse().unwrap();
        let open = Open::try_from(elem).unwrap();
        assert_eq!(open.from, None);
        assert_eq!(open.to, None);
        assert_eq!(open.id, None);
        assert_eq!(open.version, None);
        assert_eq!(open.xml_lang, None);
    }
}
