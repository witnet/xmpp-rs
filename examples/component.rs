extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::component::ComponentBuilder;
use xmpp::plugins::messaging::{MessagingPlugin, MessageEvent};

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let pass = env::var("PASS").unwrap();
    let host = env::var("HOST").unwrap();
    let port: u16 = env::var("PORT").unwrap().parse().unwrap();
    let mut component = ComponentBuilder::new(jid.clone())
                                         .password(pass)
                                         .host(host)
                                         .port(port)
                                         .connect()
                                         .unwrap();
    component.register_plugin(MessagingPlugin::new());
    loop {
        let event = component.next_event().unwrap();
        if let Some(evt) = event.downcast::<MessageEvent>() {
            println!("{:?}", evt);
        }
    }
}
