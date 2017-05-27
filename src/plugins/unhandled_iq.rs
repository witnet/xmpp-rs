use std::collections::BTreeMap;

use plugin::PluginProxy;
use event::{Priority, Propagation};

use plugins::stanza::Iq;
use xmpp_parsers::iq::IqType;
use xmpp_parsers::stanza_error::{StanzaError, ErrorType, DefinedCondition};

pub struct UnhandledIqPlugin {
    proxy: PluginProxy,
}

impl UnhandledIqPlugin {
    pub fn new() -> UnhandledIqPlugin {
        UnhandledIqPlugin {
            proxy: PluginProxy::new(),
        }
    }

    fn reply_unhandled_iq(&self, iq: &Iq) -> Propagation {
        let iq = iq.clone();
        match iq.payload {
            IqType::Get(_)
          | IqType::Set(_) => {
                self.proxy.send(Iq {
                    from: None,
                    to: Some(iq.from.unwrap()),
                    id: Some(iq.id.unwrap()),
                    payload: IqType::Error(StanzaError {
                        type_: ErrorType::Cancel,
                        defined_condition: DefinedCondition::ServiceUnavailable,
                        texts: BTreeMap::new(),
                        by: None,
                        other: None,
                    }),
                }.into());
                Propagation::Stop
            },
            IqType::Result(_)
          | IqType::Error(_) => Propagation::Continue
        }
    }
}

impl_plugin!(UnhandledIqPlugin, proxy, [
    (Iq, Priority::Min) => reply_unhandled_iq,
]);
