// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use hashes::Hash;

use minidom::{Element, IntoElements, ElementEmitter};

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
                               .attr("offset", format!("{}", self.offset))
                               .attr("length", match self.length {
                                    Some(length) => Some(format!("{}", length)),
                                    None => None
                                })
                               .build();
        for hash in self.hashes {
            elem.append_child(hash.into());
        }
        emitter.append_child(elem);
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub date: Option<String>,
    pub media_type: Option<String>,
    pub name: Option<String>,
    pub desc: Option<String>,
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
    type Error = Error;

    fn try_from(elem: Element) -> Result<Description, Error> {
        if !elem.is("description", ns::JINGLE_FT) {
            return Err(Error::ParseError("This is not a JingleFT description element."));
        }
        if elem.children().collect::<Vec<_>>().len() != 1 {
            return Err(Error::ParseError("JingleFT description element must have exactly one child."));
        }

        let mut date = None;
        let mut media_type = None;
        let mut name = None;
        let mut desc = None;
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
                    date = Some(file_payload.text());
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
                    if desc.is_some() {
                        return Err(Error::ParseError("File must not have more than one desc."));
                    }
                    desc = Some(file_payload.text());
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
                desc: desc,
                size: size,
                range: range,
                hashes: hashes,
            },
        })
    }
}

impl Into<Element> for File {
    fn into(self) -> Element {
        let mut root = Element::builder("file")
                               .ns(ns::JINGLE_FT)
                               .build();
        if let Some(date) = self.date {
            root.append_child(Element::builder("date")
                                      .ns(ns::JINGLE_FT)
                                      .append(date)
                                      .build());
        }
        if let Some(media_type) = self.media_type {
            root.append_child(Element::builder("media-type")
                                      .ns(ns::JINGLE_FT)
                                      .append(media_type)
                                      .build());
        }
        if let Some(name) = self.name {
            root.append_child(Element::builder("name")
                                      .ns(ns::JINGLE_FT)
                                      .append(name)
                                      .build());
        }
        if let Some(desc) = self.desc {
            root.append_child(Element::builder("desc")
                                      .ns(ns::JINGLE_FT)
                                      .append(desc)
                                      .build());
        }
        if let Some(size) = self.size {
            root.append_child(Element::builder("size")
                                      .ns(ns::JINGLE_FT)
                                      .append(format!("{}", size))
                                      .build());
        }
        if let Some(range) = self.range {
            root.append_child(Element::builder("range")
                                      .ns(ns::JINGLE_FT)
                                      .append(range)
                                      .build());
        }
        for hash in self.hashes {
            root.append_child(hash.into());
        }
        root
    }
}

impl Into<Element> for Description {
    fn into(self) -> Element {
        let file: Element = self.file.into();
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

    #[test]
    fn test_description() {
        let elem: Element = r#"
<description xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
  <file>
    <media-type>text/plain</media-type>
    <name>test.txt</name>
    <date>2015-07-26T21:46:00</date>
    <size>6144</size>
    <hash xmlns='urn:xmpp:hashes:2'
          algo='sha-1'>w0mcJylzCn+AfvuGdqkty2+KP48=</hash>
  </file>
</description>
"#.parse().unwrap();

        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.file.media_type, Some(String::from("text/plain")));
        assert_eq!(desc.file.name, Some(String::from("test.txt")));
        assert_eq!(desc.file.desc, None);
        assert_eq!(desc.file.date, Some(String::from("2015-07-26T21:46:00")));
        assert_eq!(desc.file.size, Some(6144u64));
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, "w0mcJylzCn+AfvuGdqkty2+KP48=");
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
        assert_eq!(desc.file.desc, None);
        assert_eq!(desc.file.date, None);
        assert_eq!(desc.file.size, None);
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, Algo::Sha_1);
        assert_eq!(desc.file.hashes[0].hash, "w0mcJylzCn+AfvuGdqkty2+KP48=");
    }
}
