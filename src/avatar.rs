// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::Event;
use futures::{sync::mpsc, Sink};
use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::{self, Write};
use tokio_xmpp::Packet;
use xmpp_parsers::{
    avatar::{Data, Metadata},
    iq::Iq,
    ns,
    pubsub::{
        event::Item,
        pubsub::{Items, PubSub},
        NodeName,
    },
    hashes::Hash,
    Jid,
};

// TODO: Update xmpp-parsers to get this function for free on Hash.
fn hash_to_hex(hash: &Hash) -> String {
    let mut bytes = vec![];
    for byte in hash.hash.iter() {
        bytes.push(format!("{:02x}", byte));
    }
    bytes.join("")
}

pub(crate) fn handle_metadata_pubsub_event(from: &Jid, tx: &mut mpsc::UnboundedSender<Packet>, items: Vec<Item>) -> impl IntoIterator<Item = Event> {
    let mut events = Vec::new();
    for item in items {
        let payload = item.payload.clone().unwrap();
        if payload.is("metadata", ns::AVATAR_METADATA) {
            let metadata = Metadata::try_from(payload).unwrap();
            for info in metadata.infos {
                let filename = format!("data/{}/{}", from, hash_to_hex(&*info.id));
                let metadata = fs::metadata(filename.clone()).unwrap();
                // TODO: Also check the hash.
                if info.bytes as u64 == metadata.len() {
                    events.push(Event::AvatarRetrieved(from.clone(), filename));
                } else {
                    let iq = download_avatar(from);
                    tx.start_send(Packet::Stanza(iq.into())).unwrap();
                }
            }
        }
    }
    events
}

fn download_avatar(from: &Jid) -> Iq {
    Iq::from_get("coucou", PubSub::Items(Items {
        max_items: None,
        node: NodeName(String::from(ns::AVATAR_DATA)),
        subid: None,
        items: Vec::new(),
    }))
    .with_to(from.clone())
}

// The return value of this function will be simply pushed to a Vec in the caller function,
// so it makes no sense to allocate a Vec here - we're lazy instead
pub(crate) fn handle_data_pubsub_iq<'a>(
    from: &'a Jid,
    items: &'a Items,
) -> impl IntoIterator<Item = Event> + 'a {
    let from = from.clone();
    items
        .items
        .iter()
        .filter_map(move |item| match (&item.id, &item.payload) {
            (Some(id), Some(payload)) => {
                let data = Data::try_from(payload.clone()).unwrap();
                let filename = save_avatar(&from, id.0.clone(), &data.data).unwrap();
                Some(Event::AvatarRetrieved(from.clone(), filename))
            }
            _ => None,
        })
}

fn save_avatar(from: &Jid, id: String, data: &[u8]) -> io::Result<String> {
    let directory = format!("data/{}", from);
    let filename = format!("data/{}/{}", from, id);
    fs::create_dir_all(directory)?;
    let mut file = File::create(&filename)?;
    file.write_all(data)?;
    Ok(filename)
}
