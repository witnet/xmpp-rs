use std::io;

use std::convert::From;

use xml::writer::Error as WriterError;
use xml::reader::Error as ReaderError;

/// An enum representing the possible errors.
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    XmlWriterError(WriterError),
    XmlReaderError(ReaderError),
    EndOfDocument,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<WriterError> for Error {
    fn from(err: WriterError) -> Error {
        Error::XmlWriterError(err)
    }
}

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::XmlReaderError(err)
    }
}
