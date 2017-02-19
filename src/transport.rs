use std::io::prelude::*;
use std::io;

use std::net::TcpStream;

use xml::reader::{EventReader, XmlEvent as XmlReaderEvent};
use xml::writer::{EventWriter, XmlEvent as XmlWriterEvent};

use std::sync::{Arc, Mutex};

use ns;

use error::Error;

use openssl::ssl::{SslMethod, SslConnectorBuilder, SslStream};

pub trait Transport {
    fn write_event<'a, E: Into<XmlWriterEvent<'a>>>(&mut self, event: E) -> Result<(), Error>;
    fn read_event(&mut self) -> Result<XmlReaderEvent, Error>;
}

struct LockedIO<T>(Arc<Mutex<T>>);

impl<T: Write> io::Write for LockedIO<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.flush()
    }
}

impl<T: Read> io::Read for LockedIO<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.read(buf)
    }
}

pub struct SslTransport {
    inner: Arc<Mutex<SslStream<TcpStream>>>, // TODO: this feels rather ugly
    reader: EventReader<LockedIO<SslStream<TcpStream>>>, // TODO: especially feels ugly because
                                                         //       this read would keep the lock
                                                         //       held very long (potentially)
    writer: EventWriter<LockedIO<SslStream<TcpStream>>>,
}

impl Transport for SslTransport {
    fn write_event<'a, E: Into<XmlWriterEvent<'a>>>(&mut self, event: E) -> Result<(), Error> {
        Ok(self.writer.write(event)?)
    }

    fn read_event(&mut self) -> Result<XmlReaderEvent, Error> {
        Ok(self.reader.next()?)
    }
}

impl SslTransport {
    pub fn connect(host: &str, port: u16) -> Result<SslTransport, Error> {
        // TODO: very quick and dirty, blame starttls
        let mut stream = TcpStream::connect((host, port))?;
        write!(stream, "<stream:stream xmlns='{}' xmlns:stream='{}' to='{}' version='1.0'>"
                     , ns::CLIENT, ns::STREAM, host)?;
        write!(stream, "<starttls xmlns='{}'/>"
                     , ns::TLS)?;
        let mut parser = EventReader::new(stream);
        loop { // TODO: possibly a timeout?
            match parser.next()? {
                XmlReaderEvent::StartElement { name, .. } => {
                    if let Some(ns) = name.namespace {
                        if ns == ns::TLS && name.local_name == "proceed" {
                            break;
                        }
                        else if ns == ns::STREAM && name.local_name == "error" {
                            return Err(Error::StreamError);
                        }
                    }
                },
                _ => {},
            }
        }
        let stream = parser.into_inner();
        let ssl_connector = SslConnectorBuilder::new(SslMethod::tls())?.build();
        let ssl_stream = Arc::new(Mutex::new(ssl_connector.connect(host, stream)?));
        let reader = EventReader::new(LockedIO(ssl_stream.clone()));
        let writer = EventWriter::new(LockedIO(ssl_stream.clone()));
        Ok(SslTransport {
            inner: ssl_stream,
            reader: reader,
            writer: writer,
        })
    }

    pub fn close(&mut self) {
        self.inner.lock()
                  .unwrap()
                  .shutdown()
                  .unwrap(); // TODO: safety, return value and such
    }
}
