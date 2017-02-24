//! Provides an `Error` for use in this crate.

use std::io;

use std::net::TcpStream;

use openssl::ssl::HandshakeError;
use openssl::error::ErrorStack;

use xml::reader::Error as XmlError;
use xml::writer::Error as EmitterError;

use minidom::Error as MinidomError;

use base64::Base64Error;

/// An error which wraps a bunch of errors from different crates and the stdlib.
#[derive(Debug)]
pub enum Error {
    XmlError(XmlError),
    EmitterError(EmitterError),
    IoError(io::Error),
    HandshakeError(HandshakeError<TcpStream>),
    OpenSslErrorStack(ErrorStack),
    MinidomError(MinidomError),
    Base64Error(Base64Error),
    SaslError(Option<String>),
    StreamError,
    EndOfDocument,
}

impl From<XmlError> for Error {
    fn from(err: XmlError) -> Error {
        Error::XmlError(err)
    }
}

impl From<EmitterError> for Error {
    fn from(err: EmitterError) -> Error {
        Error::EmitterError(err)
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

impl From<Base64Error> for Error {
    fn from(err: Base64Error) -> Error {
        Error::Base64Error(err)
    }
}
