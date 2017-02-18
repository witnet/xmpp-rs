use std::io;

use std::net::TcpStream;

use openssl::ssl::HandshakeError;
use openssl::error::ErrorStack;

use xml::reader::Error as XmlError;

#[derive(Debug)]
pub enum Error {
    XmlError(XmlError),
    IoError(io::Error),
    HandshakeError(HandshakeError<TcpStream>),
    OpenSslErrorStack(ErrorStack),
    StreamError,
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

