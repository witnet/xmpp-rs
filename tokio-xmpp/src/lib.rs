#![deny(unsafe_code, unused, missing_docs, bare_trait_objects)]

//! XMPP implementation with asynchronous I/O using Tokio.

mod starttls;
mod stream_start;
pub mod xmpp_codec;
pub use crate::xmpp_codec::Packet;
mod event;
mod happy_eyeballs;
pub mod xmpp_stream;
pub use crate::event::Event;
mod client;
pub use crate::client::Client;
mod component;
pub use crate::component::Component;
mod error;
pub use crate::error::{AuthError, ConnecterError, Error, ParseError, ParserError, ProtocolError};
