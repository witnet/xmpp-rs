use jid::Jid;
use transport::{Transport, SslTransport};
use error::Error;
use ns;

use xml::writer::XmlEvent as WriterEvent;
use xml::reader::XmlEvent as ReaderEvent;

pub struct ClientBuilder {
    jid: Jid,
    host: Option<String>,
    port: u16,
}

impl ClientBuilder {
    pub fn new(jid: Jid) -> ClientBuilder {
        ClientBuilder {
            jid: jid,
            host: None,
            port: 5222,
        }
    }

    pub fn host(mut self, host: String) -> ClientBuilder {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> ClientBuilder {
        self.port = port;
        self
    }

    pub fn connect(self) -> Result<Client, Error> {
        let host = &self.host.unwrap_or(self.jid.domain.clone());
        let mut transport = SslTransport::connect(host, self.port)?;
        transport.write_event(WriterEvent::start_element("stream:stream")
                                          .attr("to", &self.jid.domain)
                                          .default_ns(ns::CLIENT)
                                          .ns("stream", ns::STREAM))?;
        Ok(Client {
            jid: self.jid,
            transport: transport
        })
    }
}

pub struct Client {
    jid: Jid,
    transport: SslTransport,
}

impl Client {
    pub fn jid(&self) -> &Jid {
        &self.jid
    }
}
