extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::client::ClientBuilder;
use xmpp::plugins::stanza_debug::StanzaDebugPlugin;
use xmpp::plugins::stanza::StanzaPlugin;
use xmpp::plugins::unhandled_iq::UnhandledIqPlugin;
use xmpp::plugins::messaging::{MessagingPlugin, MessageEvent};
use xmpp::plugins::presence::{PresencePlugin, Type, Show};
use xmpp::plugins::disco::DiscoPlugin;
use xmpp::plugins::caps::CapsPlugin;
use xmpp::plugins::ibb::IbbPlugin;
use xmpp::plugins::ping::PingPlugin;
use xmpp::event::{Priority, Propagation};

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let pass = env::var("PASS").unwrap();
    let mut client = ClientBuilder::new(jid.clone())
                                   .password(pass)
                                   .connect()
                                   .unwrap();
    if env::var("STANZA_DEBUG").is_ok() {
        client.register_plugin(StanzaDebugPlugin::new());
    }
    client.register_plugin(StanzaPlugin::new());
    client.register_plugin(UnhandledIqPlugin::new());
    client.register_plugin(MessagingPlugin::new());
    client.register_plugin(PresencePlugin::new());
    client.register_plugin(DiscoPlugin::new("client", "bot", "en", "xmpp-rs"));
    client.register_plugin(CapsPlugin::new());
    client.register_plugin(IbbPlugin::new());
    client.register_plugin(PingPlugin::new());
    client.plugin::<PingPlugin>().init();
    client.plugin::<IbbPlugin>().init();
    client.register_handler(Priority::Max, |e: &MessageEvent| {
        println!("{:?}", e);
        Propagation::Continue
    });
    client.plugin::<PresencePlugin>().set_presence(Type::None, Show::None, None).unwrap();
    client.main().unwrap();
}
