use std::convert::TryFrom;

use plugin::PluginProxy;
use event::{Event, Priority, Propagation};
use error::Error;
use jid::Jid;

use plugins::stanza::Iq;
use xmpp_parsers::iq::{IqType, IqPayload};
use xmpp_parsers::ping::Ping;

#[derive(Debug)]
pub struct PingEvent {
    pub from: Jid,
    pub id: String,
}

impl Event for PingEvent {}

pub struct PingPlugin {
    proxy: PluginProxy,
}

impl PingPlugin {
    pub fn new() -> PingPlugin {
        PingPlugin {
            proxy: PluginProxy::new(),
        }
    }

    pub fn send_ping(&self, to: &Jid) -> Result<(), Error> {
        let to = to.clone();
        self.proxy.send(Iq {
            from: None,
            to: Some(to),
            // TODO: use a generic way to generate ids.
            id: Some(String::from("id")),
            payload: IqType::Get(IqPayload::Ping(Ping).into()),
        }.into());
        Ok(())
    }

    fn handle_iq(&self, iq: &Iq) -> Propagation {
        let iq = iq.clone();
        if let IqType::Get(payload) = iq.payload {
            // TODO: use an intermediate plugin to parse this payload.
            if let Ok(IqPayload::Ping(_)) = IqPayload::try_from(payload) {
                self.proxy.dispatch(PingEvent { // TODO: safety!!!
                    from: iq.from.unwrap(),
                    id: iq.id.unwrap(),
                });
            }
        }
        Propagation::Stop
    }

    fn reply_ping(&self, ping: &PingEvent) -> Propagation {
        self.proxy.send(Iq {
            from: None,
            to: Some(ping.from.to_owned()),
            id: Some(ping.id.to_owned()),
            payload: IqType::Result(None),
        }.into());
        Propagation::Continue
    }
}

impl_plugin!(PingPlugin, proxy, [
    (Iq, Priority::Default) => handle_iq,
    (PingEvent, Priority::Default) => reply_ping,
]);
