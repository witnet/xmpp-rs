use native_tls::Error as TlsError;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::str::Utf8Error;
use trust_dns_proto::error::ProtoError;
use trust_dns_resolver::error::ResolveError;

use xmpp_parsers::Error as ParsersError;
use xmpp_parsers::sasl::DefinedCondition as SaslDefinedCondition;

/// Top-level error type
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(IoError),
    /// Error resolving DNS and establishing a connection
    Connection(ConnecterError),
    /// DNS label conversion error, no details available from module
    /// `idna`
    Idna,
    /// Protocol-level error
    Protocol(ProtocolError),
    /// Authentication error
    Auth(AuthError),
    /// TLS error
    Tls(TlsError),
    /// Connection closed
    Disconnected,
    /// Shoud never happen
    InvalidState,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(fmt, "IO error: {}", e),
            Error::Connection(e) => write!(fmt, "connection error: {}", e),
            Error::Idna => write!(fmt, "IDNA error"),
            Error::Protocol(e) => write!(fmt, "protocol error: {}", e),
            Error::Auth(e) => write!(fmt, "authentication error: {}", e),
            Error::Tls(e) => write!(fmt, "TLS error: {}", e),
            Error::Disconnected => write!(fmt, "disconnected"),
            Error::InvalidState => write!(fmt, "invalid state"),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<ConnecterError> for Error {
    fn from(e: ConnecterError) -> Self {
        Error::Connection(e)
    }
}

impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Self {
        Error::Protocol(e)
    }
}

impl From<AuthError> for Error {
    fn from(e: AuthError) -> Self {
        Error::Auth(e)
    }
}

impl From<TlsError> for Error {
    fn from(e: TlsError) -> Self {
        Error::Tls(e)
    }
}

/// Causes for stream parsing errors
#[derive(Debug)]
pub enum ParserError {
    /// Encoding error
    Utf8(Utf8Error),
    /// XML parse error
    Parse(ParseError),
    /// Illegal `</>`
    ShortTag,
    /// Required by `impl Decoder`
    Io(IoError),
}

impl fmt::Display for ParserError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::Utf8(e) => write!(fmt, "UTF-8 error: {}", e),
            ParserError::Parse(e) => write!(fmt, "parse error: {}", e),
            ParserError::ShortTag => write!(fmt, "short tag"),
            ParserError::Io(e) => write!(fmt, "IO error: {}", e),
        }
    }
}

impl From<IoError> for ParserError {
    fn from(e: IoError) -> Self {
        ParserError::Io(e)
    }
}

impl From<ParserError> for Error {
    fn from(e: ParserError) -> Self {
        ProtocolError::Parser(e).into()
    }
}

/// XML parse error wrapper type
#[derive(Debug)]
pub struct ParseError(pub Cow<'static, str>);

impl StdError for ParseError {
    fn description(&self) -> &str {
        self.0.as_ref()
    }
    fn cause(&self) -> Option<&dyn StdError> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// XMPP protocol-level error
#[derive(Debug)]
pub enum ProtocolError {
    /// XML parser error
    Parser(ParserError),
    /// Error with expected stanza schema
    Parsers(ParsersError),
    /// No TLS available
    NoTls,
    /// Invalid response to resource binding
    InvalidBindResponse,
    /// No xmlns attribute in <stream:stream>
    NoStreamNamespace,
    /// No id attribute in <stream:stream>
    NoStreamId,
    /// Encountered an unexpected XML token
    InvalidToken,
    /// Unexpected <stream:stream> (shouldn't occur)
    InvalidStreamStart,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtocolError::Parser(e) => write!(fmt, "XML parser error: {}", e),
            ProtocolError::Parsers(e) => write!(fmt, "error with expected stanza schema: {}", e),
            ProtocolError::NoTls => write!(fmt, "no TLS available"),
            ProtocolError::InvalidBindResponse => write!(fmt, "invalid response to resource binding"),
            ProtocolError::NoStreamNamespace => write!(fmt, "no xmlns attribute in <stream:stream>"),
            ProtocolError::NoStreamId => write!(fmt, "no id attribute in <stream:stream>"),
            ProtocolError::InvalidToken => write!(fmt, "encountered an unexpected XML token"),
            ProtocolError::InvalidStreamStart => write!(fmt, "unexpected <stream:stream>"),
        }
    }
}

impl From<ParserError> for ProtocolError {
    fn from(e: ParserError) -> Self {
        ProtocolError::Parser(e)
    }
}

impl From<ParsersError> for ProtocolError {
    fn from(e: ParsersError) -> Self {
        ProtocolError::Parsers(e)
    }
}

/// Authentication error
#[derive(Debug)]
pub enum AuthError {
    /// No matching SASL mechanism available
    NoMechanism,
    /// Local SASL implementation error
    Sasl(String),
    /// Failure from server
    Fail(SaslDefinedCondition),
    /// Component authentication failure
    ComponentFail,
}

impl fmt::Display for AuthError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::NoMechanism => write!(fmt, "no matching SASL mechanism available"),
            AuthError::Sasl(s) => write!(fmt, "local SASL implementation error: {}", s),
            AuthError::Fail(c) => write!(fmt, "failure from the server: {:?}", c),
            AuthError::ComponentFail => write!(fmt, "component authentication failure"),
        }
    }
}

/// Error establishing connection
#[derive(Debug)]
pub enum ConnecterError {
    /// All attempts failed, no error available
    AllFailed,
    /// DNS protocol error
    Dns(ProtoError),
    /// DNS resolution error
    Resolve(ResolveError),
}

impl std::error::Error for ConnecterError {}

impl std::fmt::Display for ConnecterError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}
