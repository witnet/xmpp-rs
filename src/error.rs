//! Provides an error type for this crate.

use std::io;

use std::convert::From;

use xml::writer::Error as WriterError;
use xml::reader::Error as ReaderError;

/// An enum representing the possible errors.
#[derive(Debug)]
pub enum Error {
    /// An io::Error.
    IoError(io::Error),
    /// An error in the xml-rs `EventWriter`.
    XmlWriterError(WriterError),
    /// An error in the xml-rs `EventReader`.
    XmlReaderError(ReaderError),
    /// The end of the document has been reached unexpectedly.
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
