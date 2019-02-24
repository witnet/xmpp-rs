// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::error::Error;
use base64;

/// Codec for plain text content.
pub struct PlainText;

impl PlainText {
    pub fn decode(s: &str) -> Result<Option<String>, Error> {
        Ok(match s {
            "" => None,
            text => Some(text.to_owned()),
        })
    }

    pub fn encode(string: &Option<String>) -> Option<String> {
        string.as_ref().map(ToOwned::to_owned)
    }
}

/// Codec for trimmed plain text content.
pub struct TrimmedPlainText;

impl TrimmedPlainText {
    pub fn decode(s: &str) -> Result<String, Error> {
        Ok(match s.trim() {
            "" => return Err(Error::ParseError("URI missing in uri.")),
            text => text.to_owned(),
        })
    }

    pub fn encode(string: &str) -> String {
        string.to_owned()
    }
}

/// Codec wrapping base64 encode/decode.
pub struct Base64;

impl Base64 {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(s)?)
    }

    pub fn encode(b: &[u8]) -> Option<String> {
        Some(base64::encode(b))
    }
}

/// Codec wrapping base64 encode/decode, while ignoring whitespace characters.
pub struct WhitespaceAwareBase64;

impl WhitespaceAwareBase64 {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        let s: String = s.chars().filter(|ch| *ch != ' ' && *ch != '\n' && *ch != '\t').collect();
        Ok(base64::decode(&s)?)
    }

    pub fn encode(b: &[u8]) -> Option<String> {
        Some(base64::encode(b))
    }
}