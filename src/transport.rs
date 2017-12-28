//! Provides transports for the xml streams.

use std::io::BufReader;
use std::io::prelude::*;
use std::str;

use std::net::{TcpStream, Shutdown};

use quick_xml::reader::{Reader as EventReader};
use quick_xml::events::Event;

use std::sync::{Arc, Mutex};

use ns;

use minidom;

use locked_io::LockedIO;

use error::Error;

#[allow(unused_imports)]
use openssl::ssl::{SslMethod, Ssl, SslContextBuilder, SslStream, SSL_VERIFY_NONE, SslConnectorBuilder};

use sasl::common::ChannelBinding;

/// A trait which transports are required to implement.
pub trait Transport {
    /// Writes a `quick_xml::events::Event` to the stream.
    fn write_event<'a, E: Into<Event<'a>>>(&mut self, event: E) -> Result<usize, Error>;

    /// Reads a `quick_xml::events::Event` from the stream.
    fn read_event(&mut self) -> Result<Event, Error>;

    /// Writes a `minidom::Element` to the stream.
    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error>;

    /// Reads a `minidom::Element` from the stream.
    fn read_element(&mut self) -> Result<minidom::Element, Error>;

    /// Resets the stream.
    fn reset_stream(&mut self);

    /// Gets channel binding data.
    fn channel_bind(&self) -> ChannelBinding {
        ChannelBinding::None
    }
}

/// A plain text transport, completely unencrypted.
pub struct PlainTransport {
    inner: Arc<Mutex<TcpStream>>, // TODO: this feels rather ugly
    // TODO: especially feels ugly because this read would keep the lock held very long
    // (potentially)
    reader: EventReader<BufReader<LockedIO<TcpStream>>>,
    writer: LockedIO<TcpStream>,
    buf: Vec<u8>,
}

impl Transport for PlainTransport {
    fn write_event<'a, E: Into<Event<'a>>>(&mut self, event: E) -> Result<usize, Error> {
        Ok(self.writer.write(&event.into())?)
    }

    fn read_event(&mut self) -> Result<Event, Error> {
        Ok(self.reader.read_event(&mut self.buf)?)
    }

    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error> {
        Ok(element.write_to(&mut self.writer)?)
    }

    fn read_element(&mut self) -> Result<minidom::Element, Error> {
        let element = minidom::Element::from_reader(&mut self.reader)?;
        Ok(element)
    }

    fn reset_stream(&mut self) {
        let locked_io = LockedIO::from(self.inner.clone());
        self.reader = EventReader::from_reader(BufReader::new(locked_io.clone()));
        self.writer = locked_io;
    }

    fn channel_bind(&self) -> ChannelBinding {
        // TODO: channel binding
        ChannelBinding::None
    }
}

impl PlainTransport {
    /// Connects to a server without any encryption.
    pub fn connect(host: &str, port: u16) -> Result<PlainTransport, Error> {
        let tcp_stream = TcpStream::connect((host, port))?;
        let stream = Arc::new(Mutex::new(tcp_stream));
        let locked_io = LockedIO::from(stream.clone());
        let reader = EventReader::from_reader(BufReader::new(locked_io.clone()));
        let writer = locked_io;

        Ok(PlainTransport {
            inner: stream,
            reader: reader,
            writer: writer,
            buf: Vec::new(),
        })
    }

    /// Closes the stream.
    pub fn close(&mut self) {
        self.inner.lock()
                  .unwrap()
                  .shutdown(Shutdown::Both)
                  .unwrap(); // TODO: safety, return value and such
    }
}

/// A transport which uses STARTTLS.
pub struct SslTransport {
    inner: Arc<Mutex<SslStream<TcpStream>>>, // TODO: this feels rather ugly
    // TODO: especially feels ugly because this read would keep the lock held very long
    // (potentially)
    reader: EventReader<BufReader<LockedIO<SslStream<TcpStream>>>>,
    writer: LockedIO<SslStream<TcpStream>>,
    buf: Vec<u8>,
}

impl Transport for SslTransport {
    fn write_event<'a, E: Into<Event<'a>>>(&mut self, event: E) -> Result<usize, Error> {
        Ok(self.writer.write(&event.into())?)
    }

    fn read_event(&mut self) -> Result<Event, Error> {
        Ok(self.reader.read_event(&mut self.buf)?)
    }

    fn write_element(&mut self, element: &minidom::Element) -> Result<(), Error> {
        Ok(element.write_to(&mut self.writer)?)
    }

    fn read_element(&mut self) -> Result<minidom::Element, Error> {
        Ok(minidom::Element::from_reader(&mut self.reader)?)
    }

    fn reset_stream(&mut self) {
        let locked_io = LockedIO::from(self.inner.clone());
        self.reader = EventReader::from_reader(BufReader::new(locked_io.clone()));
        self.writer = locked_io;
    }

    fn channel_bind(&self) -> ChannelBinding {
        // TODO: channel binding
        ChannelBinding::None
    }
}

impl SslTransport {
    /// Connects to a server using STARTTLS.
    pub fn connect(host: &str, port: u16) -> Result<SslTransport, Error> {
        // TODO: very quick and dirty, blame starttls
        let mut stream = TcpStream::connect((host, port))?;
        write!(stream, "<stream:stream xmlns='{}' xmlns:stream='{}' to='{}' version='1.0'>"
                     , ns::CLIENT, ns::STREAM, host)?;
        write!(stream, "<starttls xmlns='{}'/>"
                     , ns::TLS)?;
        {
            let mut parser = EventReader::from_reader(BufReader::new(&stream));
            let mut buf = Vec::new();
            let ns_buf = Vec::new();
            loop { // TODO: possibly a timeout?
                match parser.read_event(&mut buf)? {
                    Event::Start(ref e) => {
                        let (namespace, local_name) = parser.resolve_namespace(e.name(), &ns_buf);
                        let namespace = namespace.map(str::from_utf8);
                        let local_name = str::from_utf8(local_name)?;

                        if let Some(ns) = namespace {
                            if ns == Ok(ns::TLS) && local_name == "proceed" {
                                break;
                            } else if ns == Ok(ns::STREAM) && local_name == "error" {
                                return Err(Error::StreamError);
                            }
                        }
                    }
                }
            }
        }
        #[cfg(feature = "insecure")]
        let ssl_stream = {
            let mut ctx = SslContextBuilder::new(SslMethod::tls())?;
            ctx.set_verify(SSL_VERIFY_NONE);
            let ssl = Ssl::new(&ctx.build())?;
            ssl.connect(stream)?
        };
        #[cfg(not(feature = "insecure"))]
        let ssl_stream = {
            let ssl_connector = SslConnectorBuilder::new(SslMethod::tls())?.build();
            ssl_connector.connect(host, stream)?
        };
        let ssl_stream = Arc::new(Mutex::new(ssl_stream));
        let locked_io = LockedIO::from(ssl_stream.clone());
        let reader = EventReader::from_reader(BufReader::new(locked_io.clone()));
        let writer = locked_io;

        Ok(SslTransport {
            inner: ssl_stream,
            reader: reader,
            writer: writer,
            buf: Vec::new(),
        })
    }

    /// Closes the stream.
    pub fn close(&mut self) {
        self.inner.lock()
                  .unwrap()
                  .shutdown()
                  .unwrap(); // TODO: safety, return value and such
    }
}
