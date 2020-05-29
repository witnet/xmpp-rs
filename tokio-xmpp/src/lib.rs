//! XMPP implementation with asynchronous I/O using Tokio.

#![deny(unsafe_code, unused, missing_docs, bare_trait_objects)]

mod starttls;
mod stream_start;
mod xmpp_codec;
pub use crate::xmpp_codec::Packet;
mod event;
mod happy_eyeballs;
pub mod xmpp_stream;
pub mod stream_features;
pub use crate::event::Event;
mod client;
pub use client::{async_client::Client as AsyncClient, simple_client::Client as SimpleClient};
mod component;
pub use crate::component::Component;
mod error;
pub use crate::error::{AuthError, ConnecterError, Error, ParseError, ParserError, ProtocolError};
