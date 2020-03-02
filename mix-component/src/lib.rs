// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_std::io;
use async_std::net::{SocketAddr, TcpStream};
use futures::io::{AsyncReadExt, AsyncWriteExt};
//use futures::channel::mpsc;
//use futures::stream::StreamExt;
use std::convert::TryFrom;
use std::ffi::CStr;
//use std::collections::hash_map::{HashMap, Entry};
use xmpp_parsers::{
    component::Handshake,
    data_forms::{DataForm, DataFormType, Field},
    disco::{
        DiscoInfoQuery, DiscoInfoResult, DiscoItemsQuery, DiscoItemsResult, Feature, Identity,
        Item as DiscoItem,
    },
    iq::{Iq, IqResultPayload, IqType},
    message::{Message, MessageType},
    mix::{
        Join as MixJoin, Leave as MixLeave, Participant as MixParticipant,
        ParticipantId as MixParticipantId, SetNick as MixSetNick, Subscribe as MixSubscribe,
        UpdateSubscription as MixUpdateSubscription,
    },
    muc::{
        user::{Affiliation, Item, Role, Status},
        MucUser,
    },
    ns,
    presence::{Presence, Type as PresenceType},
    pubsub::{pubsub::Items as PubSubItems, Item as PubSubItem, ItemId as PubSubItemId, PubSub},
    stanza_error::{DefinedCondition, ErrorType, StanzaError},
    version::{VersionQuery, VersionResult},
    BareJid, Element, FullJid, Jid,
};

mod error;
pub use error::RecvError;

/*
#[derive(Debug, PartialEq)]
pub enum Event {
    Join {
        jid: FullJid,
        room: BareJid,
    },
    Message {
        jid: FullJid,
        room: BareJid,
        body: String,
    },
}
*/

#[derive(Debug, PartialEq)]
pub enum Received {
    GetRoomsList,
}

#[derive(Debug, PartialEq)]
pub enum Send {
    RoomList(Vec<BareJid>),
}

#[derive(Debug, PartialEq)]
enum ComponentState {
    Connecting,
    Connected,
    ReceivedStreamStream { id: String },
    Authenticated {},
}

#[derive(Debug)]
pub struct Component {
    jid: BareJid,
    stream: TcpStream,
    state: ComponentState,
}

impl Component {
    pub async fn connect(
        jid: BareJid,
        server: SocketAddr,
        password: &str,
    ) -> Result<Component, RecvError> {
        let mut component = Component::new(jid, server).await?;
        component.send_stream_stream().await?;
        component.recv_stream_stream().await?;
        component.send_handshake(password).await?;
        component.recv_handshake().await?;
        Ok(component)
    }

    async fn new(jid: BareJid, server: SocketAddr) -> io::Result<Component> {
        Ok(Component {
            jid,
            stream: TcpStream::connect(server).await?,
            state: ComponentState::Connecting,
        })
    }

    async fn send_element<E: Into<Element>>(&mut self, elem: E) -> Result<(), RecvError> {
        let string = String::from(&elem.into());
        println!("SEND {}", string);
        self.stream.write_all(string.as_bytes()).await?;
        Ok(())
    }

    async fn recv_element(&mut self) -> Result<Element, RecvError> {
        let mut buf = vec![0u8; 1024];
        self.stream.read(&mut buf).await?;
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) };
        let data = c_str.to_str()?;
        println!("RECV {}", data);
        let mut elem: Element = data.parse()?;
        elem.set_ns("jabber:component:accept");
        Ok(elem)
    }

    async fn send_stream_stream(&mut self) -> io::Result<()> {
        assert_eq!(self.state, ComponentState::Connecting);
        self.state = ComponentState::Connected;
        let stream_stream = format!("<stream:stream xmlns='jabber:component:accept' xmlns:stream='http://etherx.jabber.org/streams' to='{}'>", self.jid);
        self.stream.write_all(stream_stream.as_bytes()).await?;
        println!("SEND {}", stream_stream);
        Ok(())
    }

    async fn recv_stream_stream(&mut self) -> Result<(), RecvError> {
        assert_eq!(self.state, ComponentState::Connected);
        let mut buf = vec![0u8; 1024];
        self.stream.read(&mut buf).await?;
        let data = String::from_utf8(buf)?;
        println!("RECV {}", data);
        let elem: Element = data.parse()?;
        let id = elem.attr("id").map(|i| i.to_string()).unwrap();
        self.state = ComponentState::ReceivedStreamStream { id };
        Ok(())
    }

    async fn send_handshake(&mut self, password: &str) -> Result<(), RecvError> {
        //assert_eq!(self.state, ComponentState::ReceivedStreamStream);
        match &self.state {
            ComponentState::ReceivedStreamStream { id } => {
                let handshake = Handshake::from_password_and_stream_id(password, id);
                self.send_element(handshake).await?;
                Ok(())
            }
            _ => panic!(),
        }
    }

    async fn recv_handshake(&mut self) -> Result<(), RecvError> {
        //assert_eq!(self.state, ComponentState::ReceivedStreamStream);
        let elem = self.recv_element().await?;
        if let Ok(handshake) = Handshake::try_from(elem) {
            assert!(handshake.data.is_none());
            self.state = ComponentState::Authenticated {};
            Ok(())
        } else {
            Err(RecvError::XmppParsers)
        }
    }

    fn create_disco_info(to: Jid, node: Option<String>) -> DiscoInfoResult {
        if node.is_some() {
            unimplemented!("Unknown disco#info node: {}", node.unwrap());
        }

        let mut identities = vec![];
        let mut features = vec![Feature::new(ns::DISCO_INFO)];
        if to.node().is_none() {
            // The MIX service.
            identities.push(Identity::new("conference", "mix", "en", "Rocket.chat"));
            features.extend(
                [
                    Feature::new(ns::MIX_CORE),
                    Feature::new(ns::MIX_CORE_SEARCHABLE),
                ]
                .iter()
                .cloned(),
            );
        } else {
            // A MIX channel.
            identities.push(Identity::new(
                "conference",
                "mix",
                "en",
                "Some channel’s name",
            ));
            features.extend(
                [Feature::new(ns::MIX_CORE), Feature::new(ns::MAM)]
                    .iter()
                    .cloned(),
            );
        }
        let extensions = vec![];
        DiscoInfoResult {
            node,
            identities,
            features,
            extensions,
        }
    }

    fn create_disco_items(&self, to: Jid, node: Option<String>) -> DiscoItemsResult {
        let (node, items) = match (to.clone(), node) {
            //(to, None) if to == self.jid => self.to_the_user.push(Received::GetRoomsList),
            (to, None) if to == self.jid => {
                // TODO: fetch these MIX channels from the user.
                (
                    None,
                    vec![
                        DiscoItem::new(BareJid::new("inkscape_user", self.jid.clone())),
                        DiscoItem::new(BareJid::new("team_devel", self.jid.clone())),
                    ],
                )
            }
            (to, Some(node)) if to.clone().domain() == self.jid.domain && node == "mix" => {
                // TODO: fetch these MIX channels from the user.
                (
                    Some(String::from("mix")),
                    vec![
                        DiscoItem::new(to.clone()).with_node(ns::MIX_NODES_PRESENCE),
                        DiscoItem::new(to.clone()).with_node(ns::MIX_NODES_PARTICIPANTS),
                        DiscoItem::new(to.clone()).with_node(ns::MIX_NODES_MESSAGES),
                        DiscoItem::new(to.clone()).with_node(ns::MIX_NODES_CONFIG),
                        DiscoItem::new(to.clone()).with_node(ns::MIX_NODES_INFO),
                    ],
                )
            }
            (_, _) => {
                todo!("Unknown disco#items request.");
            }
        };
        DiscoItemsResult { node, items }
    }

    fn handle_info_node(items: PubSubItems) -> PubSub {
        let node = items.node;
        let fields = vec![
            Field::text_single("Name", "MIX channel"),
            Field::text_single("Description", "Some MIX channel"),
            Field::text_single("Contact", "foo@bar.example"),
        ];
        let items = vec![xmpp_parsers::pubsub::pubsub::Item(PubSubItem {
            id: Some(PubSubItemId("2020-03-26T19:01:22+0100".into())),
            publisher: None,
            payload: Some(
                DataForm {
                    type_: DataFormType::Result_,
                    form_type: Some(ns::MIX_CORE.into()),
                    title: None,
                    instructions: None,
                    fields,
                }
                .into(),
            ),
        })];
        PubSub::Items(PubSubItems {
            node,
            items,
            max_items: None,
            subid: None,
        })
    }

    fn handle_participants_node(items: PubSubItems) -> PubSub {
        let node = items.node;
        let items = vec![
            xmpp_parsers::pubsub::pubsub::Item(PubSubItem {
                id: Some(PubSubItemId("123456".into())),
                publisher: None,
                payload: Some(
                    MixParticipant::new("hag66@shakespeare.example", "thirdwitch").into(),
                ),
            }),
            xmpp_parsers::pubsub::pubsub::Item(PubSubItem {
                id: Some(PubSubItemId("87123".into())),
                publisher: None,
                payload: Some(
                    MixParticipant::new("hecate@shakespeare.example", "top witch").into(),
                ),
            }),
        ];
        PubSub::Items(PubSubItems {
            node,
            items,
            max_items: None,
            subid: None,
        })
    }

    async fn send_iq_result(
        &mut self,
        to: FullJid,
        from: Jid,
        id: String,
        payload: Option<impl IqResultPayload>,
    ) -> Result<(), RecvError> {
        let iq = Iq::from_result(id, payload)
            .with_from(from)
            .with_to(to.into());
        self.send_element(iq).await
    }

    async fn send_iq_error(
        &mut self,
        to: FullJid,
        from: Jid,
        id: String,
        error: StanzaError,
    ) -> Result<(), RecvError> {
        let iq = Iq::from_error(id, error).with_from(from).with_to(to.into());
        self.send_element(iq).await
    }

    async fn send_join(&mut self, to: FullJid, room_plus_nick: FullJid) -> Result<(), RecvError> {
        let status = vec![Status::SelfPresence, Status::AssignedNick];
        let items = vec![Item::new(Affiliation::Member, Role::Participant)];
        let muc = MucUser { status, items };
        let presence = Presence::new(PresenceType::None)
            .with_from(room_plus_nick)
            .with_to(to)
            .with_payloads(vec![muc.into()]);
        self.send_element(presence).await
    }

    async fn send_leave(&mut self, to: FullJid, room_plus_nick: FullJid) -> Result<(), RecvError> {
        let presence = Presence::new(PresenceType::Unavailable)
            .with_to(to)
            .with_from(room_plus_nick);
        self.send_element(presence).await
    }

    async fn send_subject(
        &mut self,
        to: FullJid,
        room_plus_nick: FullJid,
    ) -> Result<(), RecvError> {
        let mut message = Message::new(Some(to.into()));
        message.from = Some(Jid::Full(room_plus_nick));
        message.add_subject("fr", "Coucou !");
        message.type_ = MessageType::Groupchat;
        self.send_element(message).await
    }

    pub async fn accept_loop(&mut self) -> Result<(), RecvError> {
        loop {
            let elem = self.recv_element().await?;
            let from: FullJid = elem.attr("from").unwrap().parse().unwrap();
            let to: Jid = elem.attr("to").unwrap().parse().unwrap();
            let id = elem.attr("id").unwrap().to_string();
            if let Ok(iq) = Iq::try_from(elem.clone()) {
                let payload = iq.payload.clone();
                match payload.clone() {
                    IqType::Get(payload) => {
                        if let Ok(disco_info) = DiscoInfoQuery::try_from(payload.clone()) {
                            let disco = Component::create_disco_info(to.clone(), disco_info.node);
                            self.send_iq_result(from, to, id, Some(disco)).await?;
                        } else if let Ok(disco_items) = DiscoItemsQuery::try_from(payload.clone()) {
                            let disco =
                                Component::create_disco_items(&self, to.clone(), disco_items.node);
                            self.send_iq_result(from, to, id, Some(disco)).await?;
                        } else if let Ok(_version) = VersionQuery::try_from(payload.clone()) {
                            let name = "Rocket.chat";
                            let version = "1.0";
                            let os = "Linux";
                            let version = VersionResult::new(name, version).with_os(os);
                            self.send_iq_result(from, to, id, Some(version)).await?;
                        } else if let Ok(pubsub) = PubSub::try_from(payload.clone()) {
                            match pubsub {
                                PubSub::Items(items) if items.node.0 == ns::MIX_NODES_INFO => {
                                    let pubsub = Component::handle_info_node(items);
                                    self.send_iq_result(from, to, id, Some(pubsub)).await?;
                                }
                                PubSub::Items(items)
                                    if items.node.0 == ns::MIX_NODES_PARTICIPANTS =>
                                {
                                    let pubsub = Component::handle_participants_node(items);
                                    self.send_iq_result(from, to, id, Some(pubsub)).await?;
                                }
                                _ => todo!("Unknown PubSub request."),
                            }
                        } else {
                            let error = StanzaError::new(
                                ErrorType::Cancel,
                                DefinedCondition::ServiceUnavailable,
                                "en",
                                "No handler for this iq…",
                            );
                            self.send_iq_error(from, to, id, error).await?;
                        }
                    }
                    IqType::Set(payload) => {
                        if let Ok(join) = MixJoin::try_from(payload.clone()) {
                            assert!(join.id.is_none());
                            let nick = join.nick;
                            let nodes = join
                                .subscribes
                                .iter()
                                .cloned()
                                .map(|subscribe| subscribe.node)
                                .collect::<Vec<_>>();
                            println!("join {} as {} {:?}", to, nick, nodes);
                            // TODO: do the join, get a Stable Participant ID, and return it to the
                            // user.
                            let join = MixJoin {
                                id: Some(MixParticipantId::new("foo")),
                                nick,
                                subscribes: nodes
                                    .iter()
                                    .cloned()
                                    .map(|node| MixSubscribe { node })
                                    .collect(),
                            };
                            self.send_iq_result(from, to, id, Some(join)).await?;
                        } else if let Ok(update_sub) =
                            MixUpdateSubscription::try_from(payload.clone())
                        {
                            let nodes = update_sub
                                .subscribes
                                .iter()
                                .cloned()
                                .map(|subscribe| subscribe.node)
                                .collect::<Vec<_>>();
                            println!("update-sub for {}: {:?}", to, nodes);
                            // TODO: do the thing.
                            let update_sub = MixUpdateSubscription {
                                jid: Some(from.clone().into()),
                                subscribes: nodes
                                    .iter()
                                    .cloned()
                                    .map(|node| MixSubscribe { node })
                                    .collect(),
                            };
                            self.send_iq_result(from, to, id, Some(update_sub)).await?;
                        } else if let Ok(_leave) = MixLeave::try_from(payload.clone()) {
                            // TODO: do the leave.
                            self.send_iq_result(from, to, id, Some(MixLeave)).await?;
                        } else if let Ok(set_nick) = MixSetNick::try_from(payload.clone()) {
                            let nick = set_nick.nick;
                            println!("set nick for {} to {}", to, nick);
                            // TODO: do the thing.
                            let set_nick = MixSetNick::new(nick);
                            self.send_iq_result(from, to, id, Some(set_nick)).await?;
                        } else {
                            let error = StanzaError::new(
                                ErrorType::Cancel,
                                DefinedCondition::ServiceUnavailable,
                                "en",
                                "No handler for this iq…",
                            );
                            self.send_iq_error(from, to, id, error).await?;
                        }
                    }
                    IqType::Result(_payload) => {}
                    IqType::Error(_error) => {}
                }
            } else if let Ok(presence) = Presence::try_from(elem.clone()) {
                match presence.type_ {
                    PresenceType::None => {
                        if let Jid::Full(room_plus_nick) = to {
                            let mut is_join = false;
                            for payload in presence.payloads.iter() {
                                if payload.is("x", ns::MUC) {
                                    is_join = true;
                                }
                            }
                            if is_join {
                                self.send_join(from.clone(), room_plus_nick.clone()).await?;
                                self.send_subject(from, room_plus_nick).await?;
                            }
                        }
                    }
                    PresenceType::Unavailable => {
                        if let Jid::Full(room_plus_nick) = to {
                            self.send_leave(from, room_plus_nick).await?;
                        }
                    }
                    _ => unimplemented!(),
                }
            } else if let Ok(message) = Message::try_from(elem) {
            } else {
                panic!();
            }
        }
    }
}

/*
pub async fn broker_loop(mut events: mpsc::Receiver<Event>) -> Result<(), RecvError> {
    let mut users: HashMap<FullJid, mpsc::UnboundedSender<String>> = HashMap::new();
    while let Some(event) = events.next().await {
        match event {
            Event::Join { jid, room } => {
                match users.entry(jid.clone()) {
                    Entry::Occupied(..) => (),
                    Entry::Vacant(entry) => {
                        let (client_sender, client_receiver) = mpsc::unbounded();
                        entry.insert(client_sender);
                        client_receiver
                    }
                }
                println!("{} join {}", jid, room);
            },
            Event::Message { jid, room, body } => println!("{} send {} to {}", jid, body, room),
        }
    }
    Ok(())
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
