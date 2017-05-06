//! A crate parsing common XMPP elements into Rust structures.
//!
//! Each module implements a `parse` function, which takes a minidom
//! `Element` reference and returns `Some(MessagePayload)` if the parsing
//! succeeded, None otherwise.
//!
//! Parsed structs can then be manipulated internally, and serialised back
//! before being sent over the wire.

// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(try_from)]

extern crate minidom;
extern crate jid;
extern crate base64;
extern crate digest;
extern crate sha2;
extern crate sha3;
extern crate blake2;

/// Error type returned by every parser on failure.
pub mod error;
/// XML namespace definitions used through XMPP.
pub mod ns;

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod message;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod presence;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod iq;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod stanza_error;

/// XEP-0004: Data Forms
pub mod data_forms;

/// XEP-0030: Service Discovery
pub mod disco;

/// XEP-0047: In-Band Bytestreams
pub mod ibb;

/// XEP-0059: Result Set Management
pub mod rsm;

/// XEP-0085: Chat State Notifications
pub mod chatstates;

/// XEP-0166: Jingle
pub mod jingle;

/// XEP-0184: Message Delivery Receipts
pub mod receipts;

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

/// XEP-0359: Unique and Stable Stanza IDs
pub mod stanza_id;

/// XEP-0380: Explicit Message Encryption
pub mod eme;

/// XEP-0390: Entity Capabilities 2.0
pub mod ecaps2;
