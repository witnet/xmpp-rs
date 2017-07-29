// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use minidom::Element;
use error::Error;
use ns;

#[derive(Debug, Clone)]
pub struct Version {
    pub name: String,
    pub version: String,
    pub os: Option<String>,
}

impl TryFrom<Element> for Version {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Version, Error> {
        if !elem.is("query", ns::VERSION) {
            return Err(Error::ParseError("This is not a version element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown child in version element."));
        }
        let mut name = None;
        let mut version = None;
        let mut os = None;
        for child in elem.children() {
            if child.is("name", ns::VERSION) {
                if name.is_some() {
                    return Err(Error::ParseError("More than one name in version element."));
                }
                name = Some(child.text());
            } else if child.is("version", ns::VERSION) {
                if version.is_some() {
                    return Err(Error::ParseError("More than one version in version element."));
                }
                version = Some(child.text());
            } else if child.is("os", ns::VERSION) {
                if os.is_some() {
                    return Err(Error::ParseError("More than one os in version element."));
                }
                os = Some(child.text());
            } else {
                return Err(Error::ParseError("Unknown child in version element."));
            }
        }
        let name = name.unwrap();
        let version = version.unwrap();
        Ok(Version {
            name,
            version,
            os,
        })
    }
}

impl From<Version> for Element {
    fn from(version: Version) -> Element {
        Element::builder("query")
                .ns(ns::VERSION)
                .append(Element::builder("name")
                                .ns(ns::VERSION)
                                .append(version.name)
                                .build())
                .append(Element::builder("version")
                                .ns(ns::VERSION)
                                .append(version.version)
                                .build())
                .append(Element::builder("os")
                                .ns(ns::VERSION)
                                .append(version.os)
                                .build())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='jabber:iq:version'><name>xmpp-rs</name><version>0.3.0</version></query>".parse().unwrap();
        let version = Version::try_from(elem).unwrap();
        assert_eq!(version.name, String::from("xmpp-rs"));
        assert_eq!(version.version, String::from("0.3.0"));
        assert_eq!(version.os, None);
    }
}
