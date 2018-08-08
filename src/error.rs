// Copyright (c) 2017-2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::From;
use std::num;
use std::string;
use std::fmt;
use std::net;

use base64;
use jid;
use chrono;

#[derive(Debug)]
pub enum Error {
    ParseError(&'static str),
    Base64Error(base64::DecodeError),
    ParseIntError(num::ParseIntError),
    ParseStringError(string::ParseError),
    ParseAddrError(net::AddrParseError),
    JidParseError(jid::JidParseError),
    ChronoParseError(chrono::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(s) => write!(fmt, "{}", s),
            Error::Base64Error(ref e) => write!(fmt, "{}", e),
            Error::ParseIntError(ref e) => write!(fmt, "{}", e),
            Error::ParseStringError(ref e) => write!(fmt, "{}", e),
            Error::ParseAddrError(ref e) => write!(fmt, "{}", e),
            Error::JidParseError(_) => write!(fmt, "JID parse error"),
            Error::ChronoParseError(ref e) => write!(fmt, "{}", e),
        }
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

impl From<net::AddrParseError> for Error {
    fn from(err: net::AddrParseError) -> Error {
        Error::ParseAddrError(err)
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
