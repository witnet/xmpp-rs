use std::convert::From;
use std::io;
use std::num;

use base64;
use minidom;
use jid;

#[derive(Debug)]
pub enum Error {
    ParseError(&'static str),
    IoError(io::Error),
    XMLError(minidom::Error),
    Base64Error(base64::DecodeError),
    ParseIntError(num::ParseIntError),
    JidParseError(jid::JidParseError),
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

impl From<jid::JidParseError> for Error {
    fn from(err: jid::JidParseError) -> Error {
        Error::JidParseError(err)
    }
}
