//! Provides an error type for this crate.

use std::io;

use std::convert::From;

use xml::writer::Error as WriterError;
use xml::reader::Error as ReaderError;

error_chain! {
    foreign_links {
        XmlWriterError(WriterError)
            /// An error with writing an XML event, from xml::writer::EventWriter.
        ;
        XmlReaderError(ReaderError)
            /// An error with reading an XML event, from xml::reader::EventReader.
        ;
        IoError(io::Error)
            /// An I/O error, from std::io.
        ;
    }

    errors {
        /// En error which is returned when the end of the document was reached prematurely.
        EndOfDocument {
            description("the end of the document has been reached prematurely")
            display("the end of the document has been reached prematurely")
        }
    }
}
