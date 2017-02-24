extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::client::ClientBuilder;
use xmpp::plugins::messaging::{MessagingPlugin, MessageEvent};
use xmpp::plugins::presence::{PresencePlugin, Show};
use xmpp::sasl::mechanisms::Plain;

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let mut client = ClientBuilder::new(jid.clone()).connect().unwrap();
    client.register_plugin(MessagingPlugin::new());
    client.register_plugin(PresencePlugin::new());
    let pass = env::var("PASS").unwrap();
    client.connect(&mut Plain::new(jid.node.clone().expect("JID requires a node"), pass)).unwrap();
    client.plugin::<PresencePlugin>().set_presence(Show::Available, None).unwrap();
    loop {
        let event = client.next_event().unwrap();
        if let Some(evt) = event.downcast::<MessageEvent>() {
            println!("{:?}", evt);
        }
    }
}
