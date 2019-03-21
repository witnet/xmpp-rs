// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use futures::{Sink, sync::mpsc};
use std::fs::{create_dir_all, File};
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
    Jid, TryFrom,
};
use crate::Event;

pub(crate) fn handle_metadata_pubsub_event(from: &Jid, tx: &mut mpsc::UnboundedSender<Packet>, items: Vec<Item>) {
    for item in items {
        let payload = item.payload.clone().unwrap();
        if payload.is("metadata", ns::AVATAR_METADATA) {
            // TODO: do something with these metadata.
            let _metadata = Metadata::try_from(payload).unwrap();
            let iq = download_avatar(from);
            tx.start_send(Packet::Stanza(iq.into())).unwrap();
        }
    }
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

pub(crate) fn handle_data_pubsub_iq(from: &Jid, tx: &mut mpsc::UnboundedSender<Event>, items: Items) {
    for item in items.items {
        if let Some(id) = item.id.clone() {
            if let Some(payload) = &item.payload {
                let data = Data::try_from(payload.clone()).unwrap();
                let filename = save_avatar(from, id.0, &data.data).unwrap();
                tx.unbounded_send(Event::AvatarRetrieved(from.clone(), filename)).unwrap();
            }
        }
    }
}

fn save_avatar(from: &Jid, id: String, data: &[u8]) -> io::Result<String> {
    let directory = format!("data/{}", from);
    let filename = format!("data/{}/{}", from, id);
    create_dir_all(directory)?;
    let mut file = File::create(&filename)?;
    file.write_all(data)?;
    Ok(filename)
}
