use jid::Jid;
use transport::{Transport, PlainTransport};
use error::Error;
use ns;
use plugin::{Plugin, PluginProxyBinding};
use event::AbstractEvent;
use connection::{Connection, Component2S};
use openssl::hash::{hash, MessageDigest};

use minidom::Element;

use xml::reader::XmlEvent as ReaderEvent;

use std::sync::mpsc::{Receiver, channel};

/// A builder for `Component`s.
pub struct ComponentBuilder {
    jid: Jid,
    secret: String,
    host: Option<String>,
    port: u16,
}

impl ComponentBuilder {
    /// Creates a new builder for an XMPP component that will connect to `jid` with default parameters.
    pub fn new(jid: Jid) -> ComponentBuilder {
        ComponentBuilder {
            jid: jid,
            secret: "".to_owned(),
            host: None,
            port: 5347,
        }
    }

    /// Sets the host to connect to.
    pub fn host(mut self, host: String) -> ComponentBuilder {
        self.host = Some(host);
        self
    }

    /// Sets the port to connect to.
    pub fn port(mut self, port: u16) -> ComponentBuilder {
        self.port = port;
        self
    }

    /// Sets the password to use.
    pub fn password<P: Into<String>>(mut self, password: P) -> ComponentBuilder {
        self.secret = password.into();
        self
    }

    /// Connects to the server and returns a `Component` when succesful.
    pub fn connect(self) -> Result<Component, Error> {
        let host = &self.host.unwrap_or(self.jid.domain.clone());
        let mut transport = PlainTransport::connect(host, self.port)?;
        Component2S::init(&mut transport, &self.jid.domain, "stream_opening")?;
        let (sender_out, sender_in) = channel();
        let (dispatcher_out, dispatcher_in) = channel();
        let mut component = Component {
            jid: self.jid,
            transport: transport,
            plugins: Vec::new(),
            binding: PluginProxyBinding::new(sender_out, dispatcher_out),
            sender_in: sender_in,
            dispatcher_in: dispatcher_in,
        };
        component.connect(self.secret)?;
        Ok(component)
    }
}

/// An XMPP component.
pub struct Component {
    jid: Jid,
    transport: PlainTransport,
    plugins: Vec<Box<Plugin>>,
    binding: PluginProxyBinding,
    sender_in: Receiver<Element>,
    dispatcher_in: Receiver<AbstractEvent>,
}

impl Component {
    /// Returns a reference to the `Jid` associated with this `Component`.
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

    fn connect(&mut self, secret: String) -> Result<(), Error> {
        // TODO: this is very ugly
        let mut sid = String::new();
        loop {
            let e = self.transport.read_event()?;
            match e {
                ReaderEvent::StartElement { attributes, .. } => {
                    for attribute in attributes {
                        if attribute.name.namespace == None && attribute.name.local_name == "id" {
                            sid = attribute.value;
                        }
                    }
                    break;
                },
                _ => (),
            }
        }
        let concatenated = format!("{}{}", sid, secret);
        let hash = hash(MessageDigest::sha1(), concatenated.as_bytes())?;
        let mut handshake = String::new();
        for byte in hash {
            // TODO: probably terrible perfs!
            handshake = format!("{}{:x}", handshake, byte);
        }
        let mut elem = Element::builder("handshake")
                               .ns(ns::COMPONENT_ACCEPT)
                               .build();
        elem.append_text_node(handshake);
        self.transport.write_element(&elem)?;
        loop {
            let n = self.transport.read_element()?;
            if n.is("handshake", ns::COMPONENT_ACCEPT) {
                return Ok(());
            }
        }
    }
}
