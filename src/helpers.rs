// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use base64;
use error::Error;

/// Codec wrapping base64 encode/decode
pub struct Base64;

impl Base64 {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(s)?)
    }

    pub fn encode(b: &Vec<u8>) -> String {
        base64::encode(b)
    }
}
