//! A crate parsing common XMPP elements into Rust structures.
//!
//! Each module implements the [`TryFrom<Element>`] trait, which takes a
//! minidom [`Element`] and returns a `Result` whose value is `Ok` if the
//! element parsed correctly, `Err(error::Error)` otherwise.
//!
//! The returned structure can be manipuled as any Rust structure, with each
//! field being public.  You can also create the same structure manually, with
//! some having `new()` and `with_*()` helper methods to create them.
//!
//! Once you are happy with your structure, you can serialise it back to an
//! [`Element`], using either `From` or `Into<Element>`, which give you what
//! you want to be sending on the wire.
//!
//! [`TryFrom<Element>`]: ../try_from/trait.TryFrom.html
//! [`Element`]: ../minidom/element/struct.Element.html

// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate minidom;
extern crate jid;
extern crate base64;
extern crate digest;
extern crate sha_1;
extern crate sha2;
extern crate sha3;
extern crate blake2;
extern crate chrono;
extern crate try_from;

/// Error type returned by every parser on failure.
pub mod error;
/// XML namespace definitions used through XMPP.
pub mod ns;
/// Helper macros to parse and serialise more easily.
#[macro_use]
pub mod macros;

#[cfg(test)]
/// Namespace-aware comparison for tests
mod compare_elements;

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod message;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod presence;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod iq;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod stanza_error;

/// RFC 6121: Extensible Messaging and Presence Protocol (XMPP): Instant Messaging and Presence
pub mod roster;

/// XEP-0004: Data Forms
pub mod data_forms;

/// XEP-0030: Service Discovery
pub mod disco;

/// XEP-0045: Multi-User Chat
pub mod muc;

/// XEP-0047: In-Band Bytestreams
pub mod ibb;

/// XEP-0059: Result Set Management
pub mod rsm;

/// XEP-0060: Publish-Subscribe
pub mod pubsub;

/// XEP-0077: In-Band Registration
pub mod ibr;

/// XEP-0082: XMPP Date and Time Profiles
pub mod date;

/// XEP-0085: Chat State Notifications
pub mod chatstates;

/// XEP-0092: Software Version
pub mod version;

/// XEP-0115: Entity Capabilities
pub mod caps;

/// XEP-0166: Jingle
pub mod jingle;

/// XEP-0184: Message Delivery Receipts
pub mod receipts;

/// XEP-0191: Blocking Command
pub mod blocking;

/// XEP-0199: XMPP Ping
pub mod ping;

/// XEP-0203: Delayed Delivery
pub mod delay;

/// XEP-0221: Data Forms Media Element
pub mod media_element;

/// XEP-0224: Attention
pub mod attention;

/// XEP-0234: Jingle File Transfer
pub mod jingle_ft;

/// XEP-0260: Jingle SOCKS5 Bytestreams Transport Method
pub mod jingle_s5b;

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub mod jingle_ibb;

/// XEP-0297: Stanza Forwarding
pub mod forwarding;

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub mod hashes;

/// XEP-0308: Last Message Correction
pub mod message_correct;

/// XEP-0313: Message Archive Management
pub mod mam;

/// XEP-0319: Last User Interaction in Presence
pub mod idle;

/// XEP-0353: Jingle Message Initiation
pub mod jingle_message;

/// XEP-0359: Unique and Stable Stanza IDs
pub mod stanza_id;

/// XEP-0380: Explicit Message Encryption
pub mod eme;

/// XEP-0390: Entity Capabilities 2.0
pub mod ecaps2;
