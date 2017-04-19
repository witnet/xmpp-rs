extern crate minidom;

pub mod error;
pub mod ns;

pub mod body;
pub mod disco;
pub mod data_forms;
pub mod media_element;
pub mod ecaps2;
pub mod jingle;
pub mod ping;
pub mod chatstates;
pub mod ibb;

use std::fmt::Debug;
use minidom::Element;

pub trait MessagePayload: Debug {}

pub fn parse_message_payload(elem: &Element) -> Option<Box<MessagePayload>> {
    if let Ok(body) = body::parse_body(elem) {
        Some(Box::new(body))
    } else if let Ok(chatstate) = chatstates::parse_chatstate(elem) {
        Some(Box::new(chatstate))
    } else {
        None
    }
}
