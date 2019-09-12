// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(bare_trait_objects)]

use std::str::FromStr;
use std::rc::Rc;
use std::cell::RefCell;
use std::convert::TryFrom;
use futures::{Future,Stream, Sink, sync::mpsc};
use tokio_xmpp::{
    Client as TokioXmppClient,
    Event as TokioXmppEvent,
    Packet,
};
use xmpp_parsers::{
    bookmarks::{
        Autojoin,
        Conference as ConferenceBookmark,
        Storage as Bookmarks,
    },
    caps::{compute_disco, hash_caps, Caps},
    disco::{DiscoInfoQuery, DiscoInfoResult, Feature, Identity},
    hashes::Algo,
    iq::{Iq, IqType},
    message::{Message, MessageType, Body},
    muc::{
        Muc,
        user::{MucUser, Status},
    },
    ns,
    presence::{Presence, Type as PresenceType},
    pubsub::{
        event::PubSubEvent,
        pubsub::PubSub,
    },
    roster::{Roster, Item as RosterItem},
    stanza_error::{StanzaError, ErrorType, DefinedCondition},
    Jid, BareJid, FullJid, JidParseError,
};

mod avatar;

pub type Error = tokio_xmpp::Error;

#[derive(Debug)]
pub enum ClientType {
    Bot,
    Pc,
}

impl Default for ClientType {
    fn default() -> Self {
        ClientType::Bot
    }
}

impl ToString for ClientType {
    fn to_string(&self) -> String {
        String::from(
            match self {
                ClientType::Bot => "bot",
                ClientType::Pc => "pc",
            }
        )
    }
}

#[derive(PartialEq)]
pub enum ClientFeature {
    Avatars,
    ContactList,
    JoinRooms,
}

#[derive(Debug)]
pub enum Event {
    Online,
    Disconnected,
    ContactAdded(RosterItem),
    ContactRemoved(RosterItem),
    ContactChanged(RosterItem),
    AvatarRetrieved(Jid, String),
    OpenRoomBookmark(ConferenceBookmark),
    RoomJoined(BareJid),
    RoomLeft(BareJid),
}

#[derive(Default)]
pub struct ClientBuilder<'a> {
    jid: &'a str,
    password: &'a str,
    website: String,
    default_nick: String,
    disco: (ClientType, String),
    features: Vec<ClientFeature>,
}

impl ClientBuilder<'_> {
    pub fn new<'a>(jid: &'a str, password: &'a str) -> ClientBuilder<'a> {
        ClientBuilder {
            jid,
            password,
            website: String::from("https://gitlab.com/xmpp-rs/tokio-xmpp"),
            default_nick: String::from("xmpp-rs"),
            disco: (ClientType::default(), String::from("tokio-xmpp")),
            features: vec![],
        }
    }

    pub fn set_client(mut self, type_: ClientType, name: &str) -> Self {
        self.disco = (type_, String::from(name));
        self
    }

    pub fn set_website(mut self, url: &str) -> Self {
        self.website = String::from(url);
        self
    }

    pub fn set_default_nick(mut self, nick: &str) -> Self {
        self.default_nick = String::from(nick);
        self
    }

    pub fn enable_feature(mut self, feature: ClientFeature) -> Self {
        self.features.push(feature);
        self
    }

    fn make_disco(&self) -> DiscoInfoResult {
        let identities = vec![Identity::new("client", self.disco.0.to_string(),
                                            "en", self.disco.1.to_string())];
        let mut features = vec![
            Feature::new(ns::DISCO_INFO),
        ];
        if self.features.contains(&ClientFeature::Avatars) {
            features.push(Feature::new(format!("{}+notify", ns::AVATAR_METADATA)));
        }
        if self.features.contains(&ClientFeature::JoinRooms) {
            features.push(Feature::new(format!("{}+notify", ns::BOOKMARKS)));
        }
        DiscoInfoResult {
            node: None,
            identities,
            features,
            extensions: vec![],
        }
    }

    fn make_initial_presence(disco: &DiscoInfoResult, node: &str) -> Presence {
        let caps_data = compute_disco(disco);
        let hash = hash_caps(&caps_data, Algo::Sha_1).unwrap();
        let caps = Caps::new(node, hash);

        let mut presence = Presence::new(PresenceType::None);
        presence.add_payload(caps);
        presence
    }

    pub fn build(
        self,
    ) -> Result<(Agent, impl Stream<Item = Event, Error = tokio_xmpp::Error>), JidParseError> {
        let client = TokioXmppClient::new(self.jid, self.password)?;
        Ok(self.build_impl(client))
    }

    // This function is meant to be used for testing build
    pub(crate) fn build_impl<S>(
        self,
        stream: S,
    ) -> (Agent, impl Stream<Item = Event, Error = tokio_xmpp::Error>)
    where
        S: Stream<Item = tokio_xmpp::Event, Error = tokio_xmpp::Error>
            + Sink<SinkItem = tokio_xmpp::Packet, SinkError = tokio_xmpp::Error>,
    {
        let disco = self.make_disco();
        let node = self.website;
        let (sender_tx, sender_rx) = mpsc::unbounded();

        let client = stream;
        let (sink, stream) = client.split();

        let reader = {
            let mut sender_tx = sender_tx.clone();
            let jid = self.jid.to_owned();
            stream.map(move |event| {
                // Helper function to send an iq error.
                let mut events = Vec::new();
                let send_error = |to, id, type_, condition, text: &str| {
                    let error = StanzaError::new(type_, condition, "en", text);
                    let iq = Iq::from_error(id, error)
                        .with_to(to)
                        .into();
                    sender_tx.unbounded_send(Packet::Stanza(iq)).unwrap();
                };

                match event {
                    TokioXmppEvent::Online => {
                        let presence = ClientBuilder::make_initial_presence(&disco, &node).into();
                        let packet = Packet::Stanza(presence);
                        sender_tx.unbounded_send(packet)
                            .unwrap();
                        events.push(Event::Online);
                        // TODO: only send this when the ContactList feature is enabled.
                        let iq = Iq::from_get("roster", Roster { ver: None, items: vec![] })
                            .into();
                        sender_tx.unbounded_send(Packet::Stanza(iq)).unwrap();
                    }
                    TokioXmppEvent::Disconnected => {
                        events.push(Event::Disconnected);
                    }
                    TokioXmppEvent::Stanza(stanza) => {
                        if stanza.is("iq", "jabber:client") {
                            let iq = Iq::try_from(stanza).unwrap();
                            if let IqType::Get(payload) = iq.payload {
                                if payload.is("query", ns::DISCO_INFO) {
                                    let query = DiscoInfoQuery::try_from(payload);
                                    match query {
                                        Ok(query) => {
                                            let mut disco_info = disco.clone();
                                            disco_info.node = query.node;
                                            let iq = Iq::from_result(iq.id, Some(disco_info))
                                                .with_to(iq.from.unwrap())
                                                .into();
                                            sender_tx.unbounded_send(Packet::Stanza(iq)).unwrap();
                                        },
                                        Err(err) => {
                                            send_error(iq.from.unwrap(), iq.id, ErrorType::Modify, DefinedCondition::BadRequest, &format!("{}", err));
                                        },
                                    }
                                } else {
                                    // We MUST answer unhandled get iqs with a service-unavailable error.
                                    send_error(iq.from.unwrap(), iq.id, ErrorType::Cancel, DefinedCondition::ServiceUnavailable, "No handler defined for this kind of iq.");
                                }
                            } else if let IqType::Result(Some(payload)) = iq.payload {
                                // TODO: move private iqs like this one somewhere else, for
                                // security reasons.
                                if payload.is("query", ns::ROSTER) && iq.from.is_none() {
                                    let roster = Roster::try_from(payload).unwrap();
                                    for item in roster.items.into_iter() {
                                        events.push(Event::ContactAdded(item));
                                    }
                                } else if payload.is("pubsub", ns::PUBSUB) {
                                    let pubsub = PubSub::try_from(payload).unwrap();
                                    let from =
                                        iq.from.clone().unwrap_or_else(|| Jid::from_str(&jid).unwrap());
                                    if let PubSub::Items(items) = pubsub {
                                        if items.node.0 == ns::AVATAR_DATA {
                                            let new_events = avatar::handle_data_pubsub_iq(&from, &items);
                                            events.extend(new_events);
                                        }
                                    }
                                }
                            } else if let IqType::Set(_) = iq.payload {
                                // We MUST answer unhandled set iqs with a service-unavailable error.
                                send_error(iq.from.unwrap(), iq.id, ErrorType::Cancel, DefinedCondition::ServiceUnavailable, "No handler defined for this kind of iq.");
                            }
                        } else if stanza.is("message", "jabber:client") {
                            let message = Message::try_from(stanza).unwrap();
                            let from = message.from.clone().unwrap();
                            for child in message.payloads {
                                if child.is("event", ns::PUBSUB_EVENT) {
                                    let event = PubSubEvent::try_from(child).unwrap();
                                    if let PubSubEvent::PublishedItems { node, items } = event {
                                        if node.0 == ns::AVATAR_METADATA {
                                            let new_events = avatar::handle_metadata_pubsub_event(&from, &mut sender_tx, items);
                                            events.extend(new_events);
                                        } else if node.0 == ns::BOOKMARKS {
                                            // TODO: Check that our bare JID is the sender.
                                            assert_eq!(items.len(), 1);
                                            let item = items.clone().pop().unwrap();
                                            let payload = item.payload.clone().unwrap();
                                            let bookmarks = match Bookmarks::try_from(payload) {
                                                Ok(bookmarks) => bookmarks,
                                                // XXX: Don’t panic…
                                                Err(err) => panic!("… {}", err),
                                            };
                                            for bookmark in bookmarks.conferences {
                                                if bookmark.autojoin == Autojoin::True {
                                                    events.push(Event::OpenRoomBookmark(bookmark));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else if stanza.is("presence", "jabber:client") {
                            let presence = Presence::try_from(stanza).unwrap();
                            let from: BareJid = match presence.from.clone().unwrap() {
                                Jid::Full(FullJid { node, domain, .. }) => BareJid { node, domain },
                                Jid::Bare(bare) => bare,
                            };
                            for payload in presence.payloads.into_iter() {
                                let muc_user = match MucUser::try_from(payload) {
                                    Ok(muc_user) => muc_user,
                                    _ => continue
                                };
                                for status in muc_user.status.into_iter() {
                                    if status == Status::SelfPresence {
                                        events.push(Event::RoomJoined(from.clone()));
                                        break;
                                    }
                                }
                            }
                        } else if stanza.is("error", "http://etherx.jabber.org/streams") {
                            println!("Received a fatal stream error: {}", String::from(&stanza));
                        } else {
                            panic!("Unknown stanza: {}", String::from(&stanza));
                        }
                    }
                }

                futures::stream::iter_ok(events)
            })
            .flatten()
        };

        let sender = sender_rx
            .map_err(|e| panic!("Sink error: {:?}", e))
            .forward(sink)
            .map(|(rx, mut sink)| {
                drop(rx);
                let _ = sink.close();
                None
            });

        // TODO is this correct?
        // Some(Error) means a real error
        // None means the end of the sender stream and can be ignored
        let future = reader
            .map(Some)
            .select(sender.into_stream())
            .filter_map(|x| x);

        let agent = Agent {
            sender_tx,
            default_nick: Rc::new(RefCell::new(self.default_nick)),
        };

        (agent, future)
    }
}

#[derive(Clone)]
pub struct Agent {
    sender_tx: mpsc::UnboundedSender<Packet>,
    default_nick: Rc<RefCell<String>>,
}

impl Agent {
    pub fn join_room(&mut self, room: BareJid, nick: Option<String>, password: Option<String>,
                     lang: &str, status: &str) {
        let mut muc = Muc::new();
        if let Some(password) = password {
            muc = muc.with_password(password);
        }

        let nick = nick.unwrap_or_else(|| self.default_nick.borrow().clone());
        let room_jid = room.with_resource(nick);
        let mut presence = Presence::new(PresenceType::None)
            .with_to(Some(Jid::Full(room_jid)));
        presence.add_payload(muc);
        presence.set_status(String::from(lang), String::from(status));
        let presence = presence.into();
        self.sender_tx.unbounded_send(Packet::Stanza(presence))
            .unwrap();
    }

    pub fn send_message(&mut self, recipient: Jid, type_: MessageType, lang: &str, text: &str) {
        let mut message = Message::new(Some(recipient));
        message.type_ = type_;
        message.bodies.insert(String::from(lang), Body(String::from(text)));
        let message = message.into();
        self.sender_tx.unbounded_send(Packet::Stanza(message))
            .unwrap();
    }
}
>>>>>>> lm-master
