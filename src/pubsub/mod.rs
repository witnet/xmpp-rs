// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::Event;
use futures::sync::mpsc;
use std::convert::TryFrom;
use std::str::FromStr;
use tokio_xmpp::Packet;
use xmpp_parsers::{
    Jid, BareJid,
    Element,
    ns,
    bookmarks2::{
        Autojoin,
        Conference,
    },
    pubsub::event::PubSubEvent,
    pubsub::pubsub::PubSub,
};

pub(crate) mod avatar;

pub(crate) fn handle_event(from: &Jid, elem: Element, mut tx: &mut mpsc::UnboundedSender<Packet>) -> impl IntoIterator<Item = Event> {
    let mut events = Vec::new();
    match PubSubEvent::try_from(elem) {
        Ok(PubSubEvent::PublishedItems { node, items }) => {
            match node.0 {
                node if node == ns::AVATAR_METADATA => {
                    let new_events = avatar::handle_metadata_pubsub_event(&from, &mut tx, items);
                    events.extend(new_events);
                },
                node if node == ns::BOOKMARKS2 => {
                    // TODO: Check that our bare JID is the sender.
                    assert_eq!(items.len(), 1);
                    let item = items.clone().pop().unwrap();
                    let jid = BareJid::from_str(&item.id.clone().unwrap().0).unwrap();
                    let payload = item.payload.clone().unwrap();
                    match Conference::try_from(payload) {
                        Ok(conference) => {
                            if conference.autojoin == Autojoin::True {
                                events.push(Event::JoinRoom(jid, conference));
                            } else {
                                events.push(Event::LeaveRoom(jid));
                            }
                        },
                        Err(err) => println!("not bookmark: {}", err)
                    }
                },
                node => unimplemented!("node {}", node)
            }
        }
        Ok(PubSubEvent::RetractedItems { node, items }) => {
            match node.0 {
                node if node == ns::BOOKMARKS2 => {
                    // TODO: Check that our bare JID is the sender.
                    assert_eq!(items.len(), 1);
                    let item = items.clone().pop().unwrap();
                    let jid = BareJid::from_str(&item.0).unwrap();
                    events.push(Event::LeaveRoom(jid));
                },
                node => unimplemented!("node {}", node)
            }
        }
        Ok(PubSubEvent::Purge { node }) => {
            match node.0 {
                node if node == ns::BOOKMARKS2 => {
                    // TODO: Check that our bare JID is the sender.
                    events.push(Event::LeaveAllRooms);
                },
                node => unimplemented!("node {}", node)
            }
        }
        _ => unimplemented!()
    }
    events
}

pub(crate) fn handle_iq_result(from: &Jid, elem: Element) -> impl IntoIterator<Item = Event> {
    let mut events = Vec::new();
    let pubsub = PubSub::try_from(elem).unwrap();
    if let PubSub::Items(items) = pubsub {
        if items.node.0 == ns::AVATAR_DATA {
            let new_events = avatar::handle_data_pubsub_iq(&from, &items);
            events.extend(new_events);
        } else if items.node.0 == ns::BOOKMARKS2 {
            events.push(Event::LeaveAllRooms);
            for item in items.items {
                let item = item.0;
                let jid = BareJid::from_str(&item.id.clone().unwrap().0).unwrap();
                let payload = item.payload.clone().unwrap();
                match Conference::try_from(payload) {
                    Ok(conference) => {
                        if let Autojoin::True = conference.autojoin {
                            events.push(Event::JoinRoom(jid, conference));
                        }
                    },
                    Err(err) => panic!("Wrong payload type in bookmarks 2 item: {}", err),
                }
            }
        }
    }
    events
}
