#![feature(raw)]

extern crate xml;
extern crate openssl;
extern crate minidom;
extern crate base64;
extern crate sha_1;
pub extern crate jid;
pub extern crate sasl;

pub mod ns;
pub mod transport;
pub mod error;
pub mod client;
pub mod component;
pub mod plugin;
#[macro_use] pub mod plugin_macro;
pub mod event;
pub mod plugins;
pub mod connection;
pub mod util;
pub mod components;

mod locked_io;
