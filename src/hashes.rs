// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};

use error::Error;

use ns;
use helpers::Base64;
use base64;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Algo {
    Sha_1,
    Sha_256,
    Sha_512,
    Sha3_256,
    Sha3_512,
    Blake2b_256,
    Blake2b_512,
    Unknown(String),
}

impl FromStr for Algo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Algo, Error> {
        Ok(match s {
            "" => return Err(Error::ParseError("'algo' argument can’t be empty.")),

            "sha-1" => Algo::Sha_1,
            "sha-256" => Algo::Sha_256,
            "sha-512" => Algo::Sha_512,
            "sha3-256" => Algo::Sha3_256,
            "sha3-512" => Algo::Sha3_512,
            "blake2b-256" => Algo::Blake2b_256,
            "blake2b-512" => Algo::Blake2b_512,
            value => Algo::Unknown(value.to_owned()),
        })
    }
}

impl From<Algo> for String {
    fn from(algo: Algo) -> String {
        String::from(match algo {
            Algo::Sha_1 => "sha-1",
            Algo::Sha_256 => "sha-256",
            Algo::Sha_512 => "sha-512",
            Algo::Sha3_256 => "sha3-256",
            Algo::Sha3_512 => "sha3-512",
            Algo::Blake2b_256 => "blake2b-256",
            Algo::Blake2b_512 => "blake2b-512",
            Algo::Unknown(text) => return text,
        })
    }
}

impl IntoAttributeValue for Algo {
    fn into_attribute_value(self) -> Option<String> {
        Some(String::from(self))
    }
}

generate_element_with_text!(
    #[derive(PartialEq)]
    Hash, "hash", ns::HASHES,
    [
        algo: Algo = "algo" => required
    ],
    hash: Base64<Vec<u8>>
);

impl Hash {
    pub fn new(algo: Algo, hash: Vec<u8>) -> Hash {
        Hash {
            algo,
            hash,
        }
    }

    pub fn from_base64(algo: Algo, hash: &str) -> Result<Hash, Error> {
        Ok(Hash::new(algo, base64::decode(hash)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;

    #[test]
    fn test_simple() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>".parse().unwrap();
        let hash = Hash::try_from(elem).unwrap();
        assert_eq!(hash.algo, Algo::Sha_256);
        assert_eq!(hash.hash, base64::decode("2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=").unwrap());
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Hash::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a hash element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2'><coucou/></hash>".parse().unwrap();
        let error = Hash::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in hash element.");
    }
}
