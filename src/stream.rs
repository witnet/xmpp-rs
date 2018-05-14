// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use jid::Jid;
use error::Error;
use ns;

generate_element_with_only_attributes!(Stream, "stream", ns::STREAM, [
    from: Option<Jid> = "from" => optional,
    to: Option<Jid> = "to" => optional,
    id: Option<String> = "id" => optional,
    version: Option<String> = "version" => optional,
    xml_lang: Option<String> = "xml:lang" => optional,
]);

impl Stream {
    pub fn new(to: Jid) -> Stream {
        Stream {
            from: None,
            to: Some(to),
            id: None,
            version: Some(String::from("1.0")),
            xml_lang: None,
        }
    }

    pub fn with_from(mut self, from: Jid) -> Stream {
        self.from = Some(from);
        self
    }

    pub fn with_id(mut self, id: String) -> Stream {
        self.id = Some(id);
        self
    }

    pub fn with_lang(mut self, xml_lang: String) -> Stream {
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
    use minidom::Element;

    #[test]
    fn test_simple() {
        let elem: Element = "<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' xml:lang='en' version='1.0' id='abc' from='some-server.example'/>".parse().unwrap();
        let stream = Stream::try_from(elem).unwrap();
        assert_eq!(stream.from, Some(Jid::domain("some-server.example")));
        assert_eq!(stream.to, None);
        assert_eq!(stream.id, Some(String::from("abc")));
        assert_eq!(stream.version, Some(String::from("1.0")));
        assert_eq!(stream.xml_lang, Some(String::from("en")));
    }
}
