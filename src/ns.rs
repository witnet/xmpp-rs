// Copyright (c) 2017-2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const JABBER_CLIENT: &str = "jabber:client";
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const XMPP_STANZAS: &str = "urn:ietf:params:xml:ns:xmpp-stanzas";
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const STREAM: &str = "http://etherx.jabber.org/streams";
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const SASL: &str = "urn:ietf:params:xml:ns:xmpp-sasl";
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const BIND: &str = "urn:ietf:params:xml:ns:xmpp-bind";

/// RFC 6121: Extensible Messaging and Presence Protocol (XMPP): Instant Messaging and Presence
pub const ROSTER: &str = "jabber:iq:roster";

/// RFC 7395: An Extensible Messaging and Presence Protocol (XMPP) Subprotocol for WebSocket
pub const WEBSOCKET: &str = "urn:ietf:params:xml:ns:xmpp-framing";

/// XEP-0004: Data Forms
pub const DATA_FORMS: &str = "jabber:x:data";

/// XEP-0030: Service Discovery
pub const DISCO_INFO: &str = "http://jabber.org/protocol/disco#info";
/// XEP-0030: Service Discovery
pub const DISCO_ITEMS: &str = "http://jabber.org/protocol/disco#items";

/// XEP-0045: Multi-User Chat
pub const MUC: &str = "http://jabber.org/protocol/muc";
/// XEP-0045: Multi-User Chat
pub const MUC_USER: &str = "http://jabber.org/protocol/muc#user";

/// XEP-0047: In-Band Bytestreams
pub const IBB: &str = "http://jabber.org/protocol/ibb";

/// XEP-0059: Result Set Management
pub const RSM: &str = "http://jabber.org/protocol/rsm";

/// XEP-0060: Publish-Subscribe
pub const PUBSUB: &str = "http://jabber.org/protocol/pubsub";
/// XEP-0060: Publish-Subscribe
pub const PUBSUB_ERRORS: &str = "http://jabber.org/protocol/pubsub#errors";
/// XEP-0060: Publish-Subscribe
pub const PUBSUB_EVENT: &str = "http://jabber.org/protocol/pubsub#event";
/// XEP-0060: Publish-Subscribe
pub const PUBSUB_OWNER: &str = "http://jabber.org/protocol/pubsub#owner";

/// XEP-0077: In-Band Registration
pub const REGISTER: &str = "jabber:iq:register";

/// XEP-0085: Chat State Notifications
pub const CHATSTATES: &str = "http://jabber.org/protocol/chatstates";

/// XEP-0092: Software Version
pub const VERSION: &str = "jabber:iq:version";

/// XEP-0107: User Mood
pub const MOOD: &str = "http://jabber.org/protocol/mood";

/// XEP-0114: Jabber Component Protocol
pub const COMPONENT_ACCEPT: &str = "jabber:component:accept";

/// XEP-0115: Entity Capabilities
pub const CAPS: &str = "http://jabber.org/protocol/caps";

/// XEP-0166: Jingle
pub const JINGLE: &str = "urn:xmpp:jingle:1";

/// XEP-0184: Message Delivery Receipts
pub const RECEIPTS: &str = "urn:xmpp:receipts";

/// XEP-0191: Blocking Command
pub const BLOCKING: &str = "urn:xmpp:blocking";
/// XEP-0191: Blocking Command
pub const BLOCKING_ERRORS: &str = "urn:xmpp:blocking:errors";

/// XEP-0199: XMPP Ping
pub const PING: &str = "urn:xmpp:ping";

/// XEP-0203: Delayed Delivery
pub const DELAY: &str = "urn:xmpp:delay";

/// XEP-0221: Data Forms Media Element
pub const MEDIA_ELEMENT: &str = "urn:xmpp:media-element";

/// XEP-0224: Attention
pub const ATTENTION: &str = "urn:xmpp:attention:0";

/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT: &str = "urn:xmpp:jingle:apps:file-transfer:5";
/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT_ERROR: &str = "urn:xmpp:jingle:apps:file-transfer:errors:0";

/// XEP-0260: Jingle SOCKS5 Bytestreams Transport Method
pub const JINGLE_S5B: &str = "urn:xmpp:jingle:transports:s5b:1";

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub const JINGLE_IBB: &str = "urn:xmpp:jingle:transports:ibb:1";

/// XEP-0297: Stanza Forwarding
pub const FORWARD: &str = "urn:xmpp:forward:0";

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASHES: &str = "urn:xmpp:hashes:2";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_256: &str = "urn:xmpp:hash-function-text-names:sha-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_512: &str = "urn:xmpp:hash-function-text-names:sha-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_256: &str = "urn:xmpp:hash-function-text-names:sha3-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_512: &str = "urn:xmpp:hash-function-text-names:sha3-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_256: &str = "urn:xmpp:hash-function-text-names:id-blake2b256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_512: &str = "urn:xmpp:hash-function-text-names:id-blake2b512";

/// XEP-0308: Last Message Correction
pub const MESSAGE_CORRECT: &str = "urn:xmpp:message-correct:0";

/// XEP-0313: Message Archive Management
pub const MAM: &str = "urn:xmpp:mam:2";

/// XEP-0319: Last User Interaction in Presence
pub const IDLE: &str = "urn:xmpp:idle:1";

/// XEP-0353: Jingle Message Initiation
pub const JINGLE_MESSAGE: &str = "urn:xmpp:jingle-message:0";

/// XEP-0359: Unique and Stable Stanza IDs
pub const SID: &str = "urn:xmpp:sid:0";

/// XEP-0380: Explicit Message Encryption
pub const EME: &str = "urn:xmpp:eme:0";

/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2: &str = "urn:xmpp:caps";
/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2_OPTIMIZE: &str = "urn:xmpp:caps:optimize";

#[cfg(not(feature = "component"))]
pub const DEFAULT_NS: &str = JABBER_CLIENT;
#[cfg(feature = "component")]
pub const DEFAULT_NS: &str = COMPONENT_ACCEPT;
