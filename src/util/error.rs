// Copyright (c) 2017-2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use base64;
use chrono;
use jid;
use std::convert::From;
use std::fmt;
use std::net;
use std::num;
use std::string;

/// Contains one of the potential errors triggered while parsing an
/// [Element](../struct.Element.html) into a specialised struct.
#[derive(Debug)]
pub enum Error {
    /// The usual error when parsing something.
    ///
    /// TODO: use a structured error so the user can report it better, instead
    /// of a freeform string.
    ParseError(&'static str),

    /// Generated when some base64 content fails to decode, usually due to
    /// extra characters.
    Base64Error(base64::DecodeError),

    /// Generated when text which should be an integer fails to parse.
    ParseIntError(num::ParseIntError),

    /// Generated when text which should be a string fails to parse.
    ParseStringError(string::ParseError),

    /// Generated when text which should be an IP address (IPv4 or IPv6) fails
    /// to parse.
    ParseAddrError(net::AddrParseError),

    /// Generated when text which should be a [JID](../../jid/struct.Jid.html)
    /// fails to parse.
    JidParseError(jid::JidParseError),

    /// Generated when text which should be a
    /// [DateTime](../date/struct.DateTime.html) fails to parse.
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
