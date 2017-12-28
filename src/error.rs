//! Provides an `Error` for use in this crate.

use std::fmt::Error as FormatError;

use std::io;

use std::net::TcpStream;
use std::str::Utf8Error;

use openssl::ssl::HandshakeError;
use openssl::error::ErrorStack;

use quick_xml::errors::Error as XmlError;

use minidom::Error as MinidomError;

use base64::DecodeError;

use components::sasl_error::SaslError;

/// An error which wraps a bunch of errors from different crates and the stdlib.
#[derive(Debug)]
pub enum Error {
    XmlError(XmlError),
    IoError(io::Error),
    HandshakeError(HandshakeError<TcpStream>),
    OpenSslErrorStack(ErrorStack),
    MinidomError(MinidomError),
    Base64Error(DecodeError),
    SaslError(Option<String>),
    XmppSaslError(SaslError),
    FormatError(FormatError),
    Utf8Error(Utf8Error),
    StreamError,
    EndOfDocument,
}

impl From<XmlError> for Error {
    fn from(err: XmlError) -> Error {
        Error::XmlError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<HandshakeError<TcpStream>> for Error {
    fn from(err: HandshakeError<TcpStream>) -> Error {
        Error::HandshakeError(err)
    }
}

impl From<ErrorStack> for Error {
    fn from(err: ErrorStack) -> Error {
        Error::OpenSslErrorStack(err)
    }
}

impl From<MinidomError> for Error {
    fn from(err: MinidomError) -> Error {
        Error::MinidomError(err)
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Error {
        Error::Base64Error(err)
    }
}

impl From<FormatError> for Error {
    fn from(err: FormatError) -> Error {
        Error::FormatError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}
