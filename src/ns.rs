// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const JABBER_CLIENT: &'static str = "jabber:client";
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const XMPP_STANZAS: &'static str = "urn:ietf:params:xml:ns:xmpp-stanzas";

/// RFC 6121: Extensible Messaging and Presence Protocol (XMPP): Instant Messaging and Presence
pub const ROSTER: &'static str = "jabber:iq:roster";

/// XEP-0004: Data Forms
pub const DATA_FORMS: &'static str = "jabber:x:data";

/// XEP-0030: Service Discovery
pub const DISCO_INFO: &'static str = "http://jabber.org/protocol/disco#info";

/// XEP-0045: Multi-User Chat
pub const MUC: &'static str = "http://jabber.org/protocol/muc";

/// XEP-0047: In-Band Bytestreams
pub const IBB: &'static str = "http://jabber.org/protocol/ibb";

/// XEP-0059: Result Set Management
pub const RSM: &'static str = "http://jabber.org/protocol/rsm";

/// XEP-0085: Chat State Notifications
pub const CHATSTATES: &'static str = "http://jabber.org/protocol/chatstates";

/// XEP-0115: Entity Capabilities
pub const CAPS: &'static str = "http://jabber.org/protocol/caps";

/// XEP-0166: Jingle
pub const JINGLE: &'static str = "urn:xmpp:jingle:1";

/// XEP-0184: Message Delivery Receipts
pub const RECEIPTS: &'static str = "urn:xmpp:receipts";

/// XEP-0199: XMPP Ping
pub const PING: &'static str = "urn:xmpp:ping";

/// XEP-0203: Delayed Delivery
pub const DELAY: &'static str = "urn:xmpp:delay";

/// XEP-0221: Data Forms Media Element
pub const MEDIA_ELEMENT: &'static str = "urn:xmpp:media-element";

/// XEP-0224: Attention
pub const ATTENTION: &'static str = "urn:xmpp:attention:0";

/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT: &'static str = "urn:xmpp:jingle:apps:file-transfer:5";
/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT_ERROR: &'static str = "urn:xmpp:jingle:apps:file-transfer:errors:0";

/// XEP-0260: Jingle SOCKS5 Bytestreams Transport Method
pub const JINGLE_S5B: &'static str = "urn:xmpp:jingle:transports:s5b:1";

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub const JINGLE_IBB: &'static str = "urn:xmpp:jingle:transports:ibb:1";

/// XEP-0297: Stanza Forwarding
pub const FORWARD: &'static str = "urn:xmpp:forward:0";

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASHES: &'static str = "urn:xmpp:hashes:2";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_256: &'static str = "urn:xmpp:hash-function-text-names:sha-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_512: &'static str = "urn:xmpp:hash-function-text-names:sha-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_256: &'static str = "urn:xmpp:hash-function-text-names:sha3-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_512: &'static str = "urn:xmpp:hash-function-text-names:sha3-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_256: &'static str = "urn:xmpp:hash-function-text-names:id-blake2b256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_512: &'static str = "urn:xmpp:hash-function-text-names:id-blake2b512";

/// XEP-0308: Last Message Correction
pub const MESSAGE_CORRECT: &'static str = "urn:xmpp:message-correct:0";

/// XEP-0313: Message Archive Management
pub const MAM: &'static str = "urn:xmpp:mam:2";

/// XEP-0319: Last User Interaction in Presence
pub const IDLE: &'static str = "urn:xmpp:idle:1";

/// XEP-0359: Unique and Stable Stanza IDs
pub const SID: &'static str = "urn:xmpp:sid:0";

/// XEP-0380: Explicit Message Encryption
pub const EME: &'static str = "urn:xmpp:eme:0";

/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2: &'static str = "urn:xmpp:caps";
/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2_OPTIMIZE: &'static str = "urn:xmpp:caps:optimize";
