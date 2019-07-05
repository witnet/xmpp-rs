// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use futures::prelude::*;
use std::env::args;
use std::process::exit;
use std::str::FromStr;
use tokio::runtime::current_thread::Runtime;
use xmpp_parsers::{Jid, message::MessageType};
use xmpp::{ClientBuilder, ClientType, ClientFeature, Event};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 5 {
        println!("Usage: {} <jid> <password> <room JID> <nick>", args[0]);
        exit(1);
    }
    let jid = &args[1];
    let password = &args[2];
    let room_jid = &args[3];
    let nick: &str = &args[4];

    // tokio_core context
    let mut rt = Runtime::new().unwrap();


    // Client instance
    let (mut agent, stream) = ClientBuilder::new(jid, password)
        .set_client(ClientType::Bot, "xmpp-rs")
        .set_website("https://gitlab.com/xmpp-rs/xmpp-rs")
        .enable_feature(ClientFeature::Avatars)
        .enable_feature(ClientFeature::ContactList)
        .build()
        .unwrap();

    // We return either Some(Error) if an error was encountered
    // or None, if we were simply disconnected
    let handler = stream.map_err(Some).for_each(|evt: Event| {
        match evt {
            Event::Online => {
                println!("Online.");
                let room_jid = Jid::from_str(room_jid).unwrap().with_resource(nick);
                agent.join_room(room_jid, "en", "Yet another bot!");
            },
            Event::Disconnected => {
                println!("Disconnected.");
                return Err(None);
            },
            Event::ContactAdded(contact) => {
                println!("Contact {:?} added.", contact);
            },
            Event::ContactRemoved(contact) => {
                println!("Contact {:?} removed.", contact);
            },
            Event::ContactChanged(contact) => {
                println!("Contact {:?} changed.", contact);
            },
            Event::RoomJoined(jid) => {
                println!("Joined room {}.", jid);
                agent.send_message(jid.into_bare_jid(), MessageType::Groupchat, "en", "Hello world!");
            },
            Event::AvatarRetrieved(jid, path) => {
                println!("Received avatar for {} in {}.", jid, path);
            },
        }
        Ok(())
    });

    rt.block_on(handler).unwrap_or_else(|e| match e {
        Some(e) => println!("Error: {:?}", e),
        None => println!("Disconnected."),
    });
}
