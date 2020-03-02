use async_std::io;

#[derive(Debug)]
pub enum RecvError {
    Io(io::Error),
    Minidom(minidom::Error),
    Utf8,
    XmppParsers,
}

impl From<io::Error> for RecvError {
    fn from(err: io::Error) -> RecvError {
        RecvError::Io(err)
    }
}

impl From<minidom::Error> for RecvError {
    fn from(err: minidom::Error) -> RecvError {
        RecvError::Minidom(err)
    }
}

impl From<std::str::Utf8Error> for RecvError {
    fn from(_err: std::str::Utf8Error) -> RecvError {
        RecvError::Utf8
    }
}

impl From<std::string::FromUtf8Error> for RecvError {
    fn from(_err: std::string::FromUtf8Error) -> RecvError {
        RecvError::Utf8
    }
}
