extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::client::ClientBuilder;

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let client = ClientBuilder::new(jid).connect().unwrap();
}
