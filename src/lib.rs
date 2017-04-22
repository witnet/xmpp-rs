//! A crate parsing common XMPP elements into Rust structures.
//!
//! The main entrypoint is `parse_message_payload`, it takes a minidom
//! `Element` reference and optionally returns `Some(MessagePayload)` if the
//! parsing succeeded.
//!
//! Parsed structs can then be manipulated internally, and serialised back
//! before being sent over the wire.

extern crate minidom;
extern crate base64;
use minidom::Element;

/// Error type returned by every parser on failure.
pub mod error;
/// XML namespace definitions used through XMPP.
pub mod ns;

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod body;

/// XEP-0004: Data Forms
pub mod data_forms;

/// XEP-0030: Service Discovery
pub mod disco;

/// XEP-0047: In-Band Bytestreams
pub mod ibb;

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

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub mod hashes;

/// XEP-0308: Last Message Correction
pub mod message_correct;

/// XEP-0380: Explicit Message Encryption
pub mod eme;

/// XEP-0390: Entity Capabilities 2.0
pub mod ecaps2;

/// Lists every known payload of a `<message/>`.
#[derive(Debug)]
pub enum MessagePayload {
    Body(body::Body),
    ChatState(chatstates::ChatState),
    Receipt(receipts::Receipt),
    Delay(delay::Delay),
    Attention(attention::Attention),
    MessageCorrect(message_correct::MessageCorrect),
    ExplicitMessageEncryption(eme::ExplicitMessageEncryption),
}

/// Parse one of the payloads of a `<message/>` element, and return `Some` of a
/// `MessagePayload` if parsing it succeeded, `None` otherwise.
pub fn parse_message_payload(elem: &Element) -> Option<MessagePayload> {
    if let Ok(body) = body::parse_body(elem) {
        Some(MessagePayload::Body(body))
    } else if let Ok(chatstate) = chatstates::parse_chatstate(elem) {
        Some(MessagePayload::ChatState(chatstate))
    } else if let Ok(receipt) = receipts::parse_receipt(elem) {
        Some(MessagePayload::Receipt(receipt))
    } else if let Ok(delay) = delay::parse_delay(elem) {
        Some(MessagePayload::Delay(delay))
    } else if let Ok(attention) = attention::parse_attention(elem) {
        Some(MessagePayload::Attention(attention))
    } else if let Ok(replace) = message_correct::parse_message_correct(elem) {
        Some(MessagePayload::MessageCorrect(replace))
    } else if let Ok(eme) = eme::parse_explicit_message_encryption(elem) {
        Some(MessagePayload::ExplicitMessageEncryption(eme))
    } else {
        None
    }
}
