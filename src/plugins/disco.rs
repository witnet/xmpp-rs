use std::convert::TryFrom;
use std::sync::{Mutex, Arc};

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use jid::Jid;

use plugins::stanza::Iq;
use xmpp_parsers::iq::IqType;
use xmpp_parsers::disco::{Disco, Identity, Feature};
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::ns;

#[derive(Debug)]
pub struct DiscoInfoRequest {
    pub from: Jid,
    pub id: String,
    pub node: Option<String>,
}

#[derive(Debug)]
pub struct DiscoInfoResult {
    pub from: Jid,
    pub id: String,
    pub disco: Disco,
}

impl Event for DiscoInfoRequest {}
impl Event for DiscoInfoResult {}

pub struct DiscoPlugin {
    proxy: PluginProxy,
    cached_disco: Arc<Mutex<Disco>>,
}

impl DiscoPlugin {
    pub fn new(category: &str, type_: &str, lang: &str, name: &str) -> DiscoPlugin {
        DiscoPlugin {
            proxy: PluginProxy::new(),
            cached_disco: Arc::new(Mutex::new(Disco {
                node: None,
                identities: vec!(Identity {
                    category: category.to_owned(),
                    type_: type_.to_owned(),
                    lang: Some(lang.to_owned()),
                    name: Some(name.to_owned())
                }),
                features: vec!(Feature { var: String::from(ns::DISCO_INFO) }),
                extensions: vec!(),
            })),
        }
    }

    pub fn add_identity(&self, category: &str, type_: &str, lang: Option<&str>, name: Option<&str>) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.identities.push(Identity {
            category: category.to_owned(),
            type_: type_.to_owned(),
            lang: lang.and_then(|lang| Some(lang.to_owned())),
            name: name.and_then(|name| Some(name.to_owned())),
        });
    }

    pub fn remove_identity(&self, category: &str, type_: &str, lang: Option<&str>, name: Option<&str>) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.identities.retain(|identity| {
            identity.category != category ||
            identity.type_ != type_ ||
            identity.lang != lang.and_then(|lang| Some(lang.to_owned())) ||
            identity.name != name.and_then(|name| Some(name.to_owned()))
        });
    }

    pub fn add_feature(&self, var: &str) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.features.push(Feature { var: String::from(var) });
    }

    pub fn remove_feature(&self, var: &str) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.features.retain(|feature| feature.var != var);
    }

    pub fn add_extension(&self, extension: DataForm) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.extensions.push(extension);
    }

    pub fn remove_extension(&self, form_type: &str) {
        let mut cached_disco = self.cached_disco.lock().unwrap();
        cached_disco.extensions.retain(|extension| {
            extension.form_type != Some(form_type.to_owned())
        });
    }

    fn handle_iq(&self, iq: &Iq) -> Propagation {
        let iq = iq.clone();
        if let IqType::Get(payload) = iq.payload {
            if let Ok(disco) = Disco::try_from(payload) {
                self.proxy.dispatch(DiscoInfoRequest {
                    from: iq.from.unwrap(),
                    id: iq.id.unwrap(),
                    node: disco.node,
                });
                return Propagation::Stop;
            }
        } else if let IqType::Result(Some(payload)) = iq.payload {
            if let Ok(disco) = Disco::try_from(payload) {
                self.proxy.dispatch(DiscoInfoResult {
                    from: iq.from.unwrap(),
                    id: iq.id.unwrap(),
                    disco: disco,
                });
                return Propagation::Stop;
            }
        }
        Propagation::Continue
    }

    fn reply_disco_info(&self, request: &DiscoInfoRequest) -> Propagation {
        let payload = if request.node.is_none() {
            let cached_disco = self.cached_disco.lock().unwrap().clone();
            IqType::Result(Some(cached_disco.into()))
        } else {
            // TODO: handle the requests on nodes too.
            return Propagation::Continue;
        };
        self.proxy.send(Iq {
            from: None,
            to: Some(request.from.to_owned()),
            id: Some(request.id.to_owned()),
            payload,
        }.into());
        Propagation::Stop
    }
}

impl_plugin!(DiscoPlugin, proxy, [
    (Iq, Priority::Default) => handle_iq,
    (DiscoInfoRequest, Priority::Default) => reply_disco_info,
]);
