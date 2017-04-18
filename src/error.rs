use std::convert::From;
use std::io;

use minidom;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    XMLError(minidom::Error),
    ParseError(&'static str),
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
