// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(bare_trait_objects)]

use futures::{future::{Either, join_all, select}, pin_mut, stream::StreamExt, sink::SinkExt};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio_xmpp::{AsyncClient as TokioXmppClient, Event as TokioXmppEvent};
use xmpp_parsers::{
    Element,
    bookmarks2::Conference,
    caps::{compute_disco, hash_caps, Caps},
    disco::{DiscoInfoQuery, DiscoInfoResult, Feature, Identity},
    hashes::Algo,
    iq::{Iq, IqType},
    message::{Body, Message, MessageType},
    muc::{
        user::{MucUser, Status},
        Muc,
    },
    ns,
    presence::{Presence, Type as PresenceType},
    pubsub::pubsub::{Items, PubSub},
    roster::{Item as RosterItem, Roster},
    stanza_error::{DefinedCondition, ErrorType, StanzaError},
    BareJid, FullJid, Jid,
};
#[macro_use]
extern crate log;

//mod pubsub;

pub type Error = tokio_xmpp::Error;

#[derive(Default)]
pub struct ClientBuilder<'a> {
    jid: &'a str,
    password: &'a str,
}

impl ClientBuilder<'_> {
    pub fn new<'a>(jid: &'a str, password: &'a str) -> ClientBuilder<'a> {
        ClientBuilder {
            jid,
            password,
        }
    }

    pub fn build(self) -> Result<Agent, Error> {
        let client = TokioXmppClient::new(self.jid, self.password)?;
        Ok(self.build_impl(client)?)
    }

    // This function is meant to be used for testing build
    pub(crate) fn build_impl(self, client: TokioXmppClient) -> Result<Agent, Error> {
        let (req_tx, req_rx) = mpsc::channel(1);
        let agent = Agent {
            client,
            req_rx,
            req_tx,
        };

        Ok(agent)
    }
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Online,
    Disconnected,
    Stanza(Element),
}

enum AgentReq {
    SendStanza(Element),
    RecvStanzas(mpsc::Sender<AgentEvent>),
}

#[derive(Clone)]
pub struct AgentHandle {
    req_tx: mpsc::Sender<AgentReq>,
}

impl AgentHandle {
    /// create a channel that gets all the stanzas
    ///
    /// there is no handling of dynamic removal of listeners!
    pub async fn events(&mut self) -> mpsc::Receiver<AgentEvent> {
        let (stanzas_tx, stanzas_rx) = mpsc::channel(1);
        self.req_tx.send(AgentReq::RecvStanzas(stanzas_tx)).await;
        stanzas_rx
    }

    pub async fn send_stanza(&mut self, stanza: Element) {
        self.req_tx.send(AgentReq::SendStanza(stanza)).await;
    }
}

pub struct Agent {
    client: TokioXmppClient,
    req_rx: mpsc::Receiver<AgentReq>,
    req_tx: mpsc::Sender<AgentReq>,
}

impl Agent {
    pub fn handle(&self) -> AgentHandle {
        AgentHandle {
            req_tx: self.req_tx.clone(),
        }
    }

    pub async fn run(self) {
        let (mut sink, mut stream) = self.client.split();
        let stream_txs1: Rc<RefCell<Vec<mpsc::Sender<AgentEvent>>>> = Rc::new(RefCell::new(vec![]));
        let stream_txs2 = stream_txs1.clone();
        let recv_future = async move {
            while let Some(xmpp_event) = stream.next().await {
                let event = match xmpp_event {
                    tokio_xmpp::Event::Online { resumed: false, .. } =>
                        Some(AgentEvent::Online),
                    tokio_xmpp::Event::Disconnected(_) =>
                        Some(AgentEvent::Disconnected),
                    tokio_xmpp::Event::Stanza(e) =>
                        Some(AgentEvent::Stanza(e)),
                    _ =>
                        None,
                };
                if let Some(event) = event {
                    join_all(stream_txs1
                             .borrow_mut()
                             .iter_mut()
                             .map(|stream_tx| stream_tx.send(event.clone()))
                    ).await;
                }
            }
        };
        let mut req_rx = self.req_rx;
        let req_future = async move {
            while let Some(req) = req_rx.recv().await {
                match req {
                    AgentReq::RecvStanzas(stanzas_tx) =>
                        stream_txs2.borrow_mut().push(stanzas_tx),
                    AgentReq::SendStanza(stanza) =>
                        sink.send(tokio_xmpp::Packet::Stanza(stanza)).await.unwrap(),
                }
            }
        };
        pin_mut!(recv_future);
        pin_mut!(req_future);
        match select(recv_future, req_future).await {
            Either::Left(((), _)) => {}
            Either::Right(((), _)) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Agent, ClientBuilder, ClientFeature, ClientType, Event};
    use tokio_xmpp::AsyncClient as TokioXmppClient;

    #[tokio::test]
    async fn test_simple() {
        let client = TokioXmppClient::new("foo@bar", "meh").unwrap();

        // Client instance
        let client_builder = ClientBuilder::new("foo@bar", "meh")
            .set_client(ClientType::Bot, "xmpp-rs")
            .set_website("https://gitlab.com/xmpp-rs/xmpp-rs")
            .set_default_nick("bot")
            .enable_feature(ClientFeature::Avatars)
            .enable_feature(ClientFeature::ContactList);

        let mut agent: Agent = client_builder.build_impl(client).unwrap();

        while let Some(events) = agent.wait_for_events().await {
            assert!(match events[0] {
                Event::Disconnected => true,
                _ => false,
            });
            assert_eq!(events.len(), 1);
            break;
        }
    }
}
