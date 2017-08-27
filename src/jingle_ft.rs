// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use std::collections::BTreeMap;
use std::str::FromStr;

use hashes::Hash;

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};
use chrono::{DateTime, FixedOffset};

use error::Error;
use ns;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub offset: u64,
    pub length: Option<u64>,
    pub hashes: Vec<Hash>,
}

impl IntoElements for Range {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let mut elem = Element::builder("range")
                               .ns(ns::JINGLE_FT)
                               .attr("offset", if self.offset == 0 { None } else { Some(self.offset) })
                               .attr("length", self.length)
                               .build();
        for hash in self.hashes {
            elem.append_child(hash.into());
        }
        emitter.append_child(elem);
    }
}

type Lang = String;

generate_id!(Desc);

#[derive(Debug, Clone)]
pub struct File {
    pub date: Option<DateTime<FixedOffset>>,
    pub media_type: Option<String>,
    pub name: Option<String>,
    pub descs: BTreeMap<Lang, Desc>,
    pub size: Option<u64>,
    pub range: Option<Range>,
    pub hashes: Vec<Hash>,
}

#[derive(Debug, Clone)]
pub struct Description {
    pub file: File,
}

#[derive(Debug, Clone)]
pub enum Creator {
    Initiator,
    Responder,
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub name: String,
    pub creator: Creator,
    pub file: File,
}

#[derive(Debug, Clone)]
pub struct Received {
    pub name: String,
    pub creator: Creator,
}

impl IntoElements for Received {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        let elem = Element::builder("received")
                           .ns(ns::JINGLE_FT)
                           .attr("name", self.name)
                           .attr("creator", match self.creator {
                                Creator::Initiator => "initiator",
                                Creator::Responder => "responder",
                            })
                           .build();
        emitter.append_child(elem);
    }
}

impl TryFrom<Element> for Description {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Description, Error> {
        if !elem.is("description", ns::JINGLE_FT) {
            return Err(Error::ParseError("This is not a JingleFT description element."));
        }
        if elem.children().count() != 1 {
            return Err(Error::ParseError("JingleFT description element must have exactly one child."));
        }

        let mut date = None;
        let mut media_type = None;
        let mut name = None;
        let mut descs = BTreeMap::new();
        let mut size = None;
        let mut range = None;
        let mut hashes = vec!();
        for description_payload in elem.children() {
            if !description_payload.is("file", ns::JINGLE_FT) {
                return Err(Error::ParseError("Unknown element in JingleFT description."));
            }
            for file_payload in description_payload.children() {
                if file_payload.is("date", ns::JINGLE_FT) {
                    if date.is_some() {
                        return Err(Error::ParseError("File must not have more than one date."));
                    }
                    date = Some(file_payload.text().parse()?);
                } else if file_payload.is("media-type", ns::JINGLE_FT) {
                    if media_type.is_some() {
                        return Err(Error::ParseError("File must not have more than one media-type."));
                    }
                    media_type = Some(file_payload.text());
                } else if file_payload.is("name", ns::JINGLE_FT) {
                    if name.is_some() {
                        return Err(Error::ParseError("File must not have more than one name."));
                    }
                    name = Some(file_payload.text());
                } else if file_payload.is("desc", ns::JINGLE_FT) {
                    let lang = get_attr!(file_payload, "xml:lang", default);
                    let desc = Desc(file_payload.text());
                    if descs.insert(lang, desc).is_some() {
                        return Err(Error::ParseError("Desc element present twice for the same xml:lang."));
                    }
                } else if file_payload.is("size", ns::JINGLE_FT) {
                    if size.is_some() {
                        return Err(Error::ParseError("File must not have more than one size."));
                    }
                    size = Some(file_payload.text().parse()?);
                } else if file_payload.is("range", ns::JINGLE_FT) {
                    if range.is_some() {
                        return Err(Error::ParseError("File must not have more than one range."));
                    }
                    let offset = get_attr!(file_payload, "offset", default);
                    let length = get_attr!(file_payload, "length", optional);
                    let mut range_hashes = vec!();
                    for hash_element in file_payload.children() {
                        if !hash_element.is("hash", ns::HASHES) {
                            return Err(Error::ParseError("Unknown element in JingleFT range."));
                        }
                        range_hashes.push(Hash::try_from(hash_element.clone())?);
                    }
                    range = Some(Range {
                        offset: offset,
                        length: length,
                        hashes: range_hashes,
                    });
                } else if file_payload.is("hash", ns::HASHES) {
                    hashes.push(Hash::try_from(file_payload.clone())?);
                } else {
                    return Err(Error::ParseError("Unknown element in JingleFT file."));
                }
            }
        }

        Ok(Description {
            file: File {
                date: date,
                media_type: media_type,
                name: name,
                descs: descs,
                size: size,
                range: range,
                hashes: hashes,
            },
        })
    }
}

impl From<File> for Element {
    fn from(file: File) -> Element {
        let mut root = Element::builder("file")
                               .ns(ns::JINGLE_FT)
                               .build();
        if let Some(date) = file.date {
            root.append_child(Element::builder("date")
                                      .ns(ns::JINGLE_FT)
                                      .append(date.to_rfc3339())
                                      .build());
        }
        if let Some(media_type) = file.media_type {
            root.append_child(Element::builder("media-type")
                                      .ns(ns::JINGLE_FT)
                                      .append(media_type)
                                      .build());
        }
        if let Some(name) = file.name {
            root.append_child(Element::builder("name")
                                      .ns(ns::JINGLE_FT)
                                      .append(name)
                                      .build());
        }
        for (lang, desc) in file.descs.into_iter() {
            root.append_child(Element::builder("desc")
                                      .ns(ns::JINGLE_FT)
                                      .attr("xml:lang", lang)
                                      .append(desc.0)
                                      .build());
        }
        if let Some(size) = file.size {
            root.append_child(Element::builder("size")
                                      .ns(ns::JINGLE_FT)
                                      .append(format!("{}", size))
                                      .build());
        }
        if let Some(range) = file.range {
            root.append_child(Element::builder("range")
                                      .ns(ns::JINGLE_FT)
                                      .append(range)
                                      .build());
        }
        for hash in file.hashes {
            root.append_child(hash.into());
        }
        root
    }
}

impl From<Description> for Element {
    fn from(description: Description) -> Element {
        let file: Element = description.file.into();
        Element::builder("description")
                .ns(ns::JINGLE_FT)
                .append(file)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashes::Algo;
    use base64;

    #[test]
    fn test_description() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <name>test.txt</name>
    <date>2015-07-26T21:46:00+01:00</date>
    <size>6144</size>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.media_type, Some(String::from("text/plain")));
        assert_eq!(desc.file.name, Some(String::from("test.txt")));
        assert_eq!(desc.file.descs, BTreeMap::new());
        assert_eq!(desc.file.date, Some(DateTime::parse_from_rfc3339("2015-07-26T21:46:00+01:00").unwrap()));
        assert_eq!(desc.file.size, Some(6144u64));
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, base64::decode("w0mcJylzCn+AfvuGdqkty2+KP48=").unwrap());
    }

    #[test]
    fn test_request() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.media_type, None);
        assert_eq!(desc.file.name, None);
        assert_eq!(desc.file.descs, BTreeMap::new());
        assert_eq!(desc.file.date, None);
        assert_eq!(desc.file.size, None);
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, base64::decode("w0mcJylzCn+AfvuGdqkty2+KP48=").unwrap());
    }

    #[test]
    fn test_descs() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <desc xml:lang='fr'>Fichier secret !</desc>
    <desc xml:lang='en'>Secret file!</desc>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.descs.keys().cloned().collect::<Vec<_>>(), ["en", "fr"]);
        assert_eq!(desc.file.descs["en"], Desc(String::from("Secret file!")));
        assert_eq!(desc.file.descs["fr"], Desc(String::from("Fichier secret !")));

        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <desc xml:lang='fr'>Fichier secret !</desc>
    <desc xml:lang='fr'>Secret file!</desc>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();
        let error = Description::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Desc element present twice for the same xml:lang.");
    }
}
