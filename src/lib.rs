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
pub mod receipts;

use minidom::Element;

#[derive(Debug)]
pub enum MessagePayload {
    Body(body::Body),
    ChatState(chatstates::ChatState),
    Receipt(receipts::Receipt),
}

pub fn parse_message_payload(elem: &Element) -> Option<MessagePayload> {
    if let Ok(body) = body::parse_body(elem) {
        Some(MessagePayload::Body(body))
    } else if let Ok(chatstate) = chatstates::parse_chatstate(elem) {
        Some(MessagePayload::ChatState(chatstate))
    } else if let Ok(receipt) = receipts::parse_receipt(elem) {
        Some(MessagePayload::Receipt(receipt))
    } else {
        None
    }
}
