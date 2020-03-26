// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// TODO: validate nicks by applying the “nickname” profile of the PRECIS OpaqueString class, as
// defined in RFC 7700.

use crate::iq::{IqResultPayload, IqSetPayload};
use crate::message::MessagePayload;
use crate::pubsub::{NodeName, PubSubPayload};
use jid::BareJid;

generate_id!(
    /// The identifier a participant receives when joining a channel.
    ParticipantId
);

generate_id!(
    /// A MIX channel identifier.
    ChannelId
);

generate_element!(
    /// Represents a participant in a MIX channel, usually returned on the
    /// urn:xmpp:mix:nodes:participants PubSub node.
    Participant, "participant", MIX_CORE,
    children: [
        /// The nick of this participant.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// The bare JID of this participant.
        // TODO: should be a BareJid!
        jid: Required<String> = ("jid", MIX_CORE) => String
    ]
);

impl PubSubPayload for Participant {}

generate_element!(
    /// A node to subscribe to.
    Subscribe, "subscribe", MIX_CORE,
    attributes: [
        /// The PubSub node to subscribe to.
        node: Required<NodeName> = "node",
    ]
);

generate_element!(
    /// A request from a user’s server to join a MIX channel.
    Join, "join", MIX_CORE,
    attributes: [
        /// The participant identifier returned by the MIX service on successful join.
        id: Option<ParticipantId> = "id",
    ],
    children: [
        /// The nick requested by the user or set by the service.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// Which MIX nodes to subscribe to.
        subscribes: Vec<Subscribe> = ("subscribe", MIX_CORE) => Subscribe
    ]
);

impl IqSetPayload for Join {}
impl IqResultPayload for Join {}

generate_element!(
    /// Update a given subscription.
    UpdateSubscription, "update-subscription", MIX_CORE,
    attributes: [
        /// The JID of the user to be affected.
        // TODO: why is it not a participant id instead?
        jid: Option<BareJid> = "jid",
    ],
    children: [
        /// The list of additional nodes to subscribe to.
        // TODO: what happens when we are already subscribed?  Also, how do we unsubscribe from
        // just one?
        subscribes: Vec<Subscribe> = ("subscribe", MIX_CORE) => Subscribe
    ]
);

impl IqSetPayload for UpdateSubscription {}
impl IqResultPayload for UpdateSubscription {}

generate_empty_element!(
    /// Request to leave a given MIX channel.  It will automatically unsubscribe the user from all
    /// nodes on this channel.
    Leave,
    "leave",
    MIX_CORE
);

impl IqSetPayload for Leave {}
impl IqResultPayload for Leave {}

generate_element!(
    /// A request to change the user’s nick.
    SetNick, "setnick", MIX_CORE,
    children: [
        /// The new requested nick.
        nick: Required<String> = ("nick", MIX_CORE) => String
    ]
);

impl IqSetPayload for SetNick {}
impl IqResultPayload for SetNick {}

generate_element!(
    /// Message payload describing who actually sent the message, since unlike in MUC, all messages
    /// are sent from the channel’s JID.
    Mix, "mix", MIX_CORE,
    children: [
        /// The nick of the user who said something.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// The JID of the user who said something.
        // TODO: should be a BareJid!
        jid: Required<String> = ("jid", MIX_CORE) => String
    ]
);

impl MessagePayload for Mix {}

generate_element!(
    /// Create a new MIX channel.
    Create, "create", MIX_CORE,
    attributes: [
        /// The requested channel identifier.
        channel: Option<ChannelId> = "channel",
    ]
);

impl IqSetPayload for Create {}
impl IqResultPayload for Create {}

generate_element!(
    /// Destroy a given MIX channel.
    Destroy, "destroy", MIX_CORE,
    attributes: [
        /// The channel identifier to be destroyed.
        channel: Required<ChannelId> = "channel",
    ]
);

// TODO: section 7.3.4, example 33, doesn’t mirror the <destroy/> in the iq result unlike every
// other section so far.
impl IqSetPayload for Destroy {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn participant() {
        let elem: Element = "<participant xmlns='urn:xmpp:mix:core:1'><jid>foo@bar</jid><nick>coucou</nick></participant>"
            .parse()
            .unwrap();
        let participant = Participant::try_from(elem).unwrap();
        assert_eq!(participant.nick, "coucou");
        assert_eq!(participant.jid, "foo@bar");
    }

    #[test]
    fn join() {
        let elem: Element = "<join xmlns='urn:xmpp:mix:core:1'><subscribe node='urn:xmpp:mix:nodes:messages'/><subscribe node='urn:xmpp:mix:nodes:info'/><nick>coucou</nick></join>"
            .parse()
            .unwrap();
        let join = Join::try_from(elem).unwrap();
        assert_eq!(join.nick, "coucou");
        assert_eq!(join.id, None);
        assert_eq!(join.subscribes.len(), 2);
        assert_eq!(join.subscribes[0].node.0, "urn:xmpp:mix:nodes:messages");
        assert_eq!(join.subscribes[1].node.0, "urn:xmpp:mix:nodes:info");
    }

    #[test]
    fn update_subscription() {
        let elem: Element = "<update-subscription xmlns='urn:xmpp:mix:core:1'><subscribe node='urn:xmpp:mix:nodes:participants'/></update-subscription>"
            .parse()
            .unwrap();
        let update_subscription = UpdateSubscription::try_from(elem).unwrap();
        assert_eq!(update_subscription.jid, None);
        assert_eq!(update_subscription.subscribes.len(), 1);
        assert_eq!(
            update_subscription.subscribes[0].node.0,
            "urn:xmpp:mix:nodes:participants"
        );
    }

    #[test]
    fn leave() {
        let elem: Element = "<leave xmlns='urn:xmpp:mix:core:1'/>".parse().unwrap();
        Leave::try_from(elem).unwrap();
    }

    #[test]
    fn setnick() {
        let elem: Element = "<setnick xmlns='urn:xmpp:mix:core:1'><nick>coucou</nick></setnick>"
            .parse()
            .unwrap();
        let setnick = SetNick::try_from(elem).unwrap();
        assert_eq!(setnick.nick, "coucou");
    }

    #[test]
    fn message_mix() {
        let elem: Element =
            "<mix xmlns='urn:xmpp:mix:core:1'><jid>foo@bar</jid><nick>coucou</nick></mix>"
                .parse()
                .unwrap();
        let mix = Mix::try_from(elem).unwrap();
        assert_eq!(mix.nick, "coucou");
        assert_eq!(mix.jid, "foo@bar");
    }

    #[test]
    fn create() {
        let elem: Element = "<create xmlns='urn:xmpp:mix:core:1' channel='coucou'/>"
            .parse()
            .unwrap();
        let create = Create::try_from(elem).unwrap();
        assert_eq!(create.channel.unwrap().0, "coucou");

        let elem: Element = "<create xmlns='urn:xmpp:mix:core:1'/>".parse().unwrap();
        let create = Create::try_from(elem).unwrap();
        assert_eq!(create.channel, None);
    }

    #[test]
    fn destroy() {
        let elem: Element = "<destroy xmlns='urn:xmpp:mix:core:1' channel='coucou'/>"
            .parse()
            .unwrap();
        let destroy = Destroy::try_from(elem).unwrap();
        assert_eq!(destroy.channel.0, "coucou");
    }
}
