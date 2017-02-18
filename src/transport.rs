use std::io::prelude::*;
use std::io;

use std::net::{SocketAddr, TcpStream};

use xml::reader::{EventReader, XmlEvent};

use ns;

use error::Error;

use openssl::ssl::{SslMethod, SslConnectorBuilder, SslStream};

pub struct SslTransport {
    inner: SslStream<TcpStream>,
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
                XmlEvent::StartElement { name, namespace, .. } => {
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
        let ssl_stream = ssl_connector.connect(host, stream)?;
        Ok(SslTransport {
            inner: ssl_stream
        })
    }
}
