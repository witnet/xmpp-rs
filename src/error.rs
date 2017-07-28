//! Provides an error type for this crate.

use std::convert::From;

error_chain! {
    foreign_links {
        XmlError(::quick_xml::errors::Error)
            /// An error from quick_xml.
        ;
        Utf8Error(::std::str::Utf8Error)
            /// An UTF-8 conversion error.
        ;
        IoError(::std::io::Error)
            /// An I/O error, from std::io.
        ;
    }

    errors {
        /// An error which is returned when the end of the document was reached prematurely.
        EndOfDocument {
            description("the end of the document has been reached prematurely")
            display("the end of the document has been reached prematurely")
        }
        /// An error which is returned when an element is closed when it shouldn't be
        InvalidElementClosed {
            description("The XML is invalid, an element was wrongly closed")
            display("the XML is invalid, an element was wrongly closed")
        }
        /// An error which is returned when an elemet's name contains more than one colon
        InvalidElement {
            description("The XML element is invalid")
            display("the XML element is invalid")
        }
    }
}
