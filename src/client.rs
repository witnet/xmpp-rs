use xml;
use jid::Jid;
use transport::{Transport, SslTransport};
use error::Error;
use ns;
use plugin::{Plugin, PluginInit, PluginProxyBinding, PluginContainer, PluginRef};
use connection::{Connection, C2S};
use sasl::client::Mechanism as SaslMechanism;
use sasl::client::mechanisms::{Plain, Scram};
use sasl::common::{Credentials as SaslCredentials, Identity, Secret, ChannelBinding};
use sasl::common::scram::{Sha1, Sha256};
use components::sasl_error::SaslError;
use util::FromElement;
use event::{Event, Dispatcher, Propagation, SendElement, ReceiveElement, Priority};

use base64;

use minidom::Element;

use xml::reader::XmlEvent as ReaderEvent;

use std::sync::{Mutex, Arc};

use std::collections::HashSet;

/// Struct that should be moved somewhere else and cleaned up.
#[derive(Debug)]
pub struct StreamFeatures {
    pub sasl_mechanisms: Option<HashSet<String>>,
}

/// A builder for `Client`s.
pub struct ClientBuilder {
    jid: Jid,
    credentials: SaslCredentials,
    host: Option<String>,
    port: u16,
}

impl ClientBuilder {
    /// Creates a new builder for an XMPP client that will connect to `jid` with default parameters.
    pub fn new(jid: Jid) -> ClientBuilder {
        ClientBuilder {
            jid: jid,
            credentials: SaslCredentials::default(),
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

    /// Sets the password to use.
    pub fn password<P: Into<String>>(mut self, password: P) -> ClientBuilder {
        self.credentials = SaslCredentials {
            identity: Identity::Username(self.jid.node.clone().expect("JID has no node")),
            secret: Secret::password_plain(password),
            channel_binding: ChannelBinding::None,
        };
        self
    }

    /// Connects to the server and returns a `Client` when succesful.
    pub fn connect(self) -> Result<Client, Error> {
        let host = &self.host.unwrap_or(self.jid.domain.clone());
        let mut transport = SslTransport::connect(host, self.port)?;
        C2S::init(&mut transport, &self.jid.domain, "before_sasl")?;
        let dispatcher = Arc::new(Dispatcher::new());
        let mut credentials = self.credentials;
        credentials.channel_binding = transport.channel_bind();
        let transport = Arc::new(Mutex::new(transport));
        let plugin_container = Arc::new(PluginContainer::new());
        let mut client = Client {
            jid: self.jid.clone(),
            transport: transport.clone(),
            binding: PluginProxyBinding::new(dispatcher.clone(), plugin_container.clone(), self.jid),
            plugin_container: plugin_container,
            dispatcher: dispatcher,
        };
        client.dispatcher.register(Priority::Default, move |evt: &SendElement| {
            let mut t = transport.lock().unwrap();
            t.write_element(&evt.0).unwrap();
            Propagation::Continue
        });
        client.connect(credentials)?;
        client.bind()?;
        Ok(client)
    }
}

/// An XMPP client.
pub struct Client {
    jid: Jid,
    transport: Arc<Mutex<SslTransport>>,
    plugin_container: Arc<PluginContainer>,
    binding: PluginProxyBinding,
    dispatcher: Arc<Dispatcher>,
}

impl Client {
    /// Returns a reference to the `Jid` associated with this `Client`.
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

    fn reset_stream(&self) {
        self.transport.lock().unwrap().reset_stream()
    }

    fn read_element(&self) -> Result<Element, Error> {
        self.transport.lock().unwrap().read_element()
    }

    fn write_element(&self, elem: &Element) -> Result<(), Error> {
        self.transport.lock().unwrap().write_element(elem)
    }

    fn read_event(&self) -> Result<xml::reader::XmlEvent, Error> {
        self.transport.lock().unwrap().read_event()
    }

    fn connect(&mut self, mut credentials: SaslCredentials) -> Result<(), Error> {
        let features = self.wait_for_features()?;
        let ms = &features.sasl_mechanisms.ok_or(Error::SaslError(Some("no SASL mechanisms".to_owned())))?;
        fn wrap_err(err: String) -> Error { Error::SaslError(Some(err)) }
        // TODO: better way for selecting these, enabling anonymous auth
        let mut mechanism: Box<SaslMechanism> = if ms.contains("SCRAM-SHA-256-PLUS") && credentials.channel_binding != ChannelBinding::None {
            Box::new(Scram::<Sha256>::from_credentials(credentials).map_err(wrap_err)?)
        }
        else if ms.contains("SCRAM-SHA-1-PLUS") && credentials.channel_binding != ChannelBinding::None {
            Box::new(Scram::<Sha1>::from_credentials(credentials).map_err(wrap_err)?)
        }
        else if ms.contains("SCRAM-SHA-256") {
            if credentials.channel_binding != ChannelBinding::None {
                credentials.channel_binding = ChannelBinding::Unsupported;
            }
            Box::new(Scram::<Sha256>::from_credentials(credentials).map_err(wrap_err)?)
        }
        else if ms.contains("SCRAM-SHA-1") {
            if credentials.channel_binding != ChannelBinding::None {
                credentials.channel_binding = ChannelBinding::Unsupported;
            }
            Box::new(Scram::<Sha1>::from_credentials(credentials).map_err(wrap_err)?)
        }
        else if ms.contains("PLAIN") {
            Box::new(Plain::from_credentials(credentials).map_err(wrap_err)?)
        }
        else {
            return Err(Error::SaslError(Some("can't find a SASL mechanism to use".to_owned())));
        };
        let auth = mechanism.initial().map_err(|x| Error::SaslError(Some(x)))?;
        let mut elem = Element::builder("auth")
                               .ns(ns::SASL)
                               .attr("mechanism", mechanism.name())
                               .build();
        if !auth.is_empty() {
            elem.append_text_node(base64::encode(&auth));
        }
        self.write_element(&elem)?;
        loop {
            let n = self.read_element()?;
            if n.is("challenge", ns::SASL) {
                let text = n.text();
                let challenge = if text == "" {
                    Vec::new()
                }
                else {
                    base64::decode(&text)?
                };
                let response = mechanism.response(&challenge).map_err(|x| Error::SaslError(Some(x)))?;
                let mut elem = Element::builder("response")
                                       .ns(ns::SASL)
                                       .build();
                if !response.is_empty() {
                    elem.append_text_node(base64::encode(&response));
                }
                self.write_element(&elem)?;
            }
            else if n.is("success", ns::SASL) {
                let text = n.text();
                let data = if text == "" {
                    Vec::new()
                }
                else {
                    base64::decode(&text)?
                };
                mechanism.success(&data).map_err(|x| Error::SaslError(Some(x)))?;
                self.reset_stream();
                {
                    let mut g = self.transport.lock().unwrap();
                    C2S::init(&mut *g, &self.jid.domain, "after_sasl")?;
                }
                self.wait_for_features()?;
                return Ok(());
            }
            else if n.is("failure", ns::SASL) {
                let inner = SaslError::from_element(&n).map_err(|_| Error::SaslError(None))?;
                return Err(Error::XmppSaslError(inner));
            }
        }
    }

    fn bind(&mut self) -> Result<(), Error> {
        let mut elem = Element::builder("iq")
                               .attr("id", "bind")
                               .attr("type", "set")
                               .build();
        let mut bind = Element::builder("bind")
                               .ns(ns::BIND)
                               .build();
        if let Some(ref resource) = self.jid.resource {
            let res = Element::builder("resource")
                              .ns(ns::BIND)
                              .append(resource.to_owned())
                              .build();
            bind.append_child(res);
        }
        elem.append_child(bind);
        self.write_element(&elem)?;
        loop {
            let n = self.read_element()?;
            if n.is("iq", ns::CLIENT) && n.has_child("bind", ns::BIND) {
                return Ok(());
            }
        }
    }

    fn wait_for_features(&mut self) -> Result<StreamFeatures, Error> {
        // TODO: this is very ugly
        loop {
            let e = self.read_event()?;
            match e {
                ReaderEvent::StartElement { .. } => {
                    break;
                },
                _ => (),
            }
        }
        loop {
            let n = self.read_element()?;
            if n.is("features", ns::STREAM) {
                let mut features = StreamFeatures {
                    sasl_mechanisms: None,
                };
                if let Some(ms) = n.get_child("mechanisms", ns::SASL) {
                    let mut res = HashSet::new();
                    for cld in ms.children() {
                        res.insert(cld.text());
                    }
                    features.sasl_mechanisms = Some(res);
                }
                return Ok(features);
            }
        }
    }
}
