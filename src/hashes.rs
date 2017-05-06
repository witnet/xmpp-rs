// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone, PartialEq)]
pub struct Hash {
    pub algo: String,
    pub hash: String,
}

impl<'a> TryFrom<&'a Element> for Hash {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Hash, Error> {
        if !elem.is("hash", ns::HASHES) {
            return Err(Error::ParseError("This is not a hash element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in hash element."));
        }
        let algo = elem.attr("algo").ok_or(Error::ParseError("Mandatory argument 'algo' not present in hash element."))?.to_owned();
        let hash = match elem.text().as_ref() {
            "" => return Err(Error::ParseError("Hash element shouldn’t be empty.")),
            text => text.to_owned(),
        };
        Ok(Hash {
            algo: algo,
            hash: hash,
        })
    }
}

impl<'a> Into<Element> for &'a Hash {
    fn into(self) -> Element {
        Element::builder("hash")
                .ns(ns::HASHES)
                .attr("algo", self.algo.clone())
                .append(self.hash.clone())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>".parse().unwrap();
        let hash = Hash::try_from(&elem).unwrap();
        assert_eq!(hash.algo, "sha-256");
        assert_eq!(hash.hash, "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=");
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Hash::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a hash element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2'><coucou/></hash>".parse().unwrap();
        let error = Hash::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in hash element.");
    }
}
