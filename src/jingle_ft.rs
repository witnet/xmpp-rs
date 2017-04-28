extern crate minidom;

use hashes;
use hashes::{Hash, parse_hash};

use minidom::{Element, IntoElements};
use minidom::convert::ElementEmitter;

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
            elem.append_child(hashes::serialise(&hash));
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
pub struct Creator {
    pub creator: String,
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub name: String,
    pub creator: Creator,
    pub file: File,
}

pub fn parse_jingle_ft(root: &Element) -> Result<Description, Error> {
    if !root.is("description", ns::JINGLE_FT) {
        return Err(Error::ParseError("This is not a JingleFT description element."));
    }
    if root.children().collect::<Vec<_>>().len() != 1 {
        return Err(Error::ParseError("JingleFT description element must have exactly one child."));
    }

    let mut date = None;
    let mut media_type = None;
    let mut name = None;
    let mut desc = None;
    let mut size = None;
    let mut range = None;
    let mut hashes = vec!();
    for description_payload in root.children() {
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
                let offset = file_payload.attr("offset").unwrap_or("0").parse()?;
                let length = match file_payload.attr("length") {
                    Some(length) => Some(length.parse()?),
                    None => None,
                };
                let mut range_hashes = vec!();
                for hash_element in file_payload.children() {
                    if !hash_element.is("hash", ns::HASHES) {
                        return Err(Error::ParseError("Unknown element in JingleFT range."));
                    }
                    range_hashes.push(parse_hash(hash_element)?);
                }
                range = Some(Range {
                    offset: offset,
                    length: length,
                    hashes: range_hashes,
                });
            } else if file_payload.is("hash", ns::HASHES) {
                hashes.push(parse_hash(file_payload)?);
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

pub fn serialise_file(file: &File) -> Element {
    let mut root = Element::builder("file")
                           .ns(ns::JINGLE_FT)
                           .build();
    println!("{:#?}", file);
    if let Some(ref date) = file.date {
        root.append_child(Element::builder("date")
                                  .ns(ns::JINGLE_FT)
                                  .append(date.clone())
                                  .build());
    }
    if let Some(ref media_type) = file.media_type {
        root.append_child(Element::builder("media-type")
                                  .ns(ns::JINGLE_FT)
                                  .append(media_type.clone())
                                  .build());
    }
    if let Some(ref name) = file.name {
        root.append_child(Element::builder("name")
                                  .ns(ns::JINGLE_FT)
                                  .append(name.clone())
                                  .build());
    }
    if let Some(ref desc) = file.desc {
        root.append_child(Element::builder("desc")
                                  .ns(ns::JINGLE_FT)
                                  .append(desc.clone())
                                  .build());
    }
    if let Some(ref size) = file.size {
        root.append_child(Element::builder("size")
                                  .ns(ns::JINGLE_FT)
                                  .append(format!("{}", size))
                                  .build());
    }
    if let Some(ref range) = file.range {
        root.append_child(Element::builder("range")
                                  .ns(ns::JINGLE_FT)
                                  .append(range.clone())
                                  .build());
    }
    for hash in file.hashes.clone() {
        root.append_child(hashes::serialise(&hash));
    }
    root
}

pub fn serialise(desc: &Description) -> Element {
    Element::builder("description")
            .ns(ns::JINGLE_FT)
            .append(serialise_file(&desc.file))
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use jingle_ft;

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

        let desc = jingle_ft::parse_jingle_ft(&elem).unwrap();
        assert_eq!(desc.file.media_type, Some(String::from("text/plain")));
        assert_eq!(desc.file.name, Some(String::from("test.txt")));
        assert_eq!(desc.file.desc, None);
        assert_eq!(desc.file.date, Some(String::from("2015-07-26T21:46:00")));
        assert_eq!(desc.file.size, Some(6144u64));
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, "sha-1");
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

        let desc = jingle_ft::parse_jingle_ft(&elem).unwrap();
        assert_eq!(desc.file.media_type, None);
        assert_eq!(desc.file.name, None);
        assert_eq!(desc.file.desc, None);
        assert_eq!(desc.file.date, None);
        assert_eq!(desc.file.size, None);
        assert_eq!(desc.file.range, None);
        assert_eq!(desc.file.hashes[0].algo, "sha-1");
        assert_eq!(desc.file.hashes[0].hash, "w0mcJylzCn+AfvuGdqkty2+KP48=");
    }
}
