use jid::Jid;
use transport::{Transport, SslTransport};
use error::Error;
use ns;
use plugin::{Plugin, PluginProxyBinding};
use event::AbstractEvent;
use connection::{Connection, C2S};

use base64;

use minidom::Element;

use xml::reader::XmlEvent as ReaderEvent;

use std::sync::mpsc::{Receiver, channel};

/// A builder for `Client`s.
pub struct ClientBuilder {
    jid: Jid,
    host: Option<String>,
    port: u16,
}

impl ClientBuilder {
    /// Creates a new builder for an XMPP client that will connect to `jid` with default parameters.
    pub fn new(jid: Jid) -> ClientBuilder {
        ClientBuilder {
            jid: jid,
            host: None,
            port: 5222,
        }
    }

    /// Sets the host to connect to.
    pub fn host(mut self, host: String) -> ClientBuilder {
        self.host = Some(host);
        self
    }

    /// Sets the port to connect to.
    pub fn port(mut self, port: u16) -> ClientBuilder {
        self.port = port;
        self
    }

    /// Connects to the server and returns a `Client` when succesful.
    pub fn connect(self) -> Result<Client, Error> {
        let host = &self.host.unwrap_or(self.jid.domain.clone());
        let mut transport = SslTransport::connect(host, self.port)?;
        C2S::init(&mut transport, &self.jid.domain, "before_sasl")?;
        let (sender_out, sender_in) = channel();
        let (dispatcher_out, dispatcher_in) = channel();
        Ok(Client {
            jid: self.jid,
            transport: transport,
            plugins: Vec::new(),
            binding: PluginProxyBinding::new(sender_out, dispatcher_out),
            sender_in: sender_in,
            dispatcher_in: dispatcher_in,
        })
    }
}

/// An XMPP client.
pub struct Client {
    jid: Jid,
    transport: SslTransport,
    plugins: Vec<Box<Plugin>>,
    binding: PluginProxyBinding,
    sender_in: Receiver<Element>,
    dispatcher_in: Receiver<AbstractEvent>,
}

impl Client {
    /// Returns a reference to the `Jid` associated with this `Client`.
    pub fn jid(&self) -> &Jid {
        &self.jid
    }

    /// Registers a plugin.
    pub fn register_plugin<P: Plugin + 'static>(&mut self, mut plugin: P) {
        plugin.bind(self.binding.clone());
        self.plugins.push(Box::new(plugin));
    }

    /// Returns the plugin given by the type parameter, if it exists, else panics.
    pub fn plugin<P: Plugin>(&self) -> &P {
        for plugin in &self.plugins {
            let any = plugin.as_any();
            if let Some(ret) = any.downcast_ref::<P>() {
                return ret;
            }
        }
        panic!("plugin does not exist!");
    }

    /// Returns the next event and flush the send queue.
    pub fn next_event(&mut self) -> Result<AbstractEvent, Error> {
        self.flush_send_queue()?;
        loop {
            if let Ok(evt) = self.dispatcher_in.try_recv() {
                return Ok(evt);
            }
            let elem = self.transport.read_element()?;
            for plugin in self.plugins.iter_mut() {
                plugin.handle(&elem);
                // TODO: handle plugin return
            }
            self.flush_send_queue()?;
        }
    }

    /// Flushes the send queue, sending all queued up stanzas.
    pub fn flush_send_queue(&mut self) -> Result<(), Error> { // TODO: not sure how great of an
                                                              //       idea it is to flush in this
                                                              //       manner…
        while let Ok(elem) = self.sender_in.try_recv() {
            self.transport.write_element(&elem)?;
        }
        Ok(())
    }

    /// Connects using SASL plain authentication.
    pub fn connect_plain(&mut self, password: &str) -> Result<(), Error> {
        // TODO: this is very ugly
        loop {
            let e = self.transport.read_event().unwrap();
            match e {
                ReaderEvent::StartElement { .. } => {
                    break;
                },
                _ => (),
            }
        }
        let mut did_sasl = false;
        loop {
            let n = self.transport.read_element().unwrap();
            if n.is("features", ns::STREAM) {
                if did_sasl {
                    let mut elem = Element::builder("iq")
                                           .attr("id", "bind")
                                           .attr("type", "set")
                                           .build();
                    let bind = Element::builder("bind")
                                       .ns(ns::BIND)
                                       .build();
                    elem.append_child(bind);
                    self.transport.write_element(&elem)?;
                }
                else {
                    let mut auth = Vec::new();
                    auth.push(0);
                    auth.extend(self.jid.node.as_ref().expect("JID has no node").bytes());
                    auth.push(0);
                    auth.extend(password.bytes());
                    let elem = Element::builder("auth")
                                       .text(base64::encode(&auth))
                                       .ns(ns::SASL)
                                       .attr("mechanism", "PLAIN")
                                       .build();
                    self.transport.write_element(&elem)?;
                    did_sasl = true;
                }
            }
            else if n.is("success", ns::SASL) {
                self.transport.reset_stream();
                C2S::init(&mut self.transport, &self.jid.domain, "after_sasl")?;
                loop {
                    let e = self.transport.read_event()?;
                    match e {
                        ReaderEvent::StartElement { .. } => {
                            break;
                        },
                        _ => (),
                    }
                }
            }
            else if n.is("iq", ns::CLIENT) && n.has_child("bind", ns::BIND) {
                return Ok(());
            }
        }
    }
}
