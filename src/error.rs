// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::From;
use std::io;
use std::num;
use std::string;

use base64;
use minidom;
use jid;
use chrono;

#[derive(Debug)]
pub enum Error {
    ParseError(&'static str),
    IoError(io::Error),
    XMLError(minidom::Error),
    Base64Error(base64::DecodeError),
    ParseIntError(num::ParseIntError),
    ParseStringError(string::ParseError),
    JidParseError(jid::JidParseError),
    ChronoParseError(chrono::ParseError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<minidom::Error> for Error {
    fn from(err: minidom::Error) -> Error {
        Error::XMLError(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::Base64Error(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

impl From<string::ParseError> for Error {
    fn from(err: string::ParseError) -> Error {
        Error::ParseStringError(err)
    }
}

impl From<jid::JidParseError> for Error {
    fn from(err: jid::JidParseError) -> Error {
        Error::JidParseError(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::ChronoParseError(err)
    }
}
