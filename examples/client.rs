extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::client::ClientBuilder;
use xmpp::plugins::messaging::{MessagingPlugin, MessageEvent};
use xmpp::plugins::presence::{PresencePlugin, Show};
use xmpp::plugins::ping::{PingPlugin, PingEvent};
use xmpp::event::{Priority, Propagation};

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let pass = env::var("PASS").unwrap();
    let mut client = ClientBuilder::new(jid.clone())
                                   .password(pass)
                                   .connect()
                                   .unwrap();
    client.register_plugin(MessagingPlugin::new());
    client.register_plugin(PresencePlugin::new());
    client.register_plugin(PingPlugin::new());
    client.register_handler(Priority::Max, |e: &MessageEvent| {
        println!("{:?}", e);
        Propagation::Continue
    });
    client.plugin::<PresencePlugin>().set_presence(Show::Available, None).unwrap();
    client.main().unwrap();
}
