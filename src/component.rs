use jid::Jid;
use transport::{Transport, PlainTransport};
use error::Error;
use ns;
use plugin::{Plugin, PluginInit, PluginProxyBinding, PluginContainer, PluginRef};
use event::{Dispatcher, ReceiveElement, SendElement, Propagation, Priority, Event};
use connection::{Connection, Component2S};
use sha_1::{Sha1, Digest};

use minidom::Element;

use quick_xml::events::Event as XmlEvent;

use std::str;
use std::fmt::Write;
use std::sync::{Mutex, Arc};

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
        let dispatcher = Arc::new(Dispatcher::new());
        let transport = Arc::new(Mutex::new(transport));
        let plugin_container = Arc::new(PluginContainer::new());
        let mut component = Component {
            jid: self.jid.clone(),
            transport: transport.clone(),
            binding: PluginProxyBinding::new(dispatcher.clone(), plugin_container.clone(), self.jid),
            plugin_container: plugin_container,
            dispatcher: dispatcher,
        };
        component.dispatcher.register(Priority::Default, move |evt: &SendElement| {
            let mut t = transport.lock().unwrap();
            t.write_element(&evt.0).unwrap();
            Propagation::Continue
        });
        component.connect(self.secret)?;
        Ok(component)
    }
}

/// An XMPP component.
pub struct Component {
    jid: Jid,
    transport: Arc<Mutex<PlainTransport>>,
    plugin_container: Arc<PluginContainer>,
    binding: PluginProxyBinding,
    dispatcher: Arc<Dispatcher>,
}

impl Component {
    /// Returns a reference to the `Jid` associated with this `Component`.
    pub fn jid(&self) -> &Jid {
        &self.jid
    }

    /// Registers a plugin.
    pub fn register_plugin<P: Plugin + PluginInit + 'static>(&mut self, mut plugin: P) {
        let binding = self.binding.clone();
        plugin.bind(binding);
        let p = Arc::new(plugin);
        P::init(&self.dispatcher, p.clone());
        self.plugin_container.register(p);
    }

    pub fn register_handler<E, F>(&mut self, pri: Priority, func: F)
        where
            E: Event,
            F: Fn(&E) -> Propagation + 'static {
        self.dispatcher.register(pri, func);
    }

    /// Returns the plugin given by the type parameter, if it exists, else panics.
    pub fn plugin<P: Plugin>(&self) -> PluginRef<P> {
        self.plugin_container.get::<P>().unwrap()
    }

    /// Returns the next event and flush the send queue.
    pub fn main(&mut self) -> Result<(), Error> {
        self.dispatcher.flush_all();
        loop {
            let elem = self.read_element()?;
            self.dispatcher.dispatch(ReceiveElement(elem));
            self.dispatcher.flush_all();
        }
    }

    fn read_element(&self) -> Result<Element, Error> {
        self.transport.lock().unwrap().read_element()
    }

    fn write_element(&self, elem: &Element) -> Result<(), Error> {
        self.transport.lock().unwrap().write_element(elem)
    }

    fn connect(&mut self, secret: String) -> Result<(), Error> {
        let mut sid = String::new();
        loop {
            let mut transport = self.transport.lock().unwrap();
            let e = transport.read_event()?;
            match e {
                XmlEvent::Start(ref e) => {
                    let mut attributes = e.attributes()
                        .map(|o| {
                            let o = o?;
                            let key = str::from_utf8(o.key)?;
                            let value = str::from_utf8(&o.value)?;
                            Ok((key, value))
                            }
                        )
                        .collect::<Result<Vec<(&str, &str)>, Error>>()?;
                    for &(name, value) in &attributes {
                        if name == "id" {
                            sid = value.to_owned();
                        }
                    }
                }
            }
            break
        }
        let concatenated = format!("{}{}", sid, secret);
        let mut hasher = Sha1::default();
        hasher.input(concatenated.as_bytes());
        let mut handshake = String::new();
        for byte in hasher.result() {
            write!(handshake, "{:02x}", byte)?;
        }
        let mut elem = Element::builder("handshake")
                               .ns(ns::COMPONENT_ACCEPT)
                               .build();
        elem.append_text_node(handshake);
        self.write_element(&elem)?;
        loop {
            let n = self.read_element()?;
            if n.is("handshake", ns::COMPONENT_ACCEPT) {
                return Ok(());
            }
        }
    }
}
