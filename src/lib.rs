extern crate xml;
extern crate openssl;
extern crate minidom;
extern crate base64;
pub extern crate jid;

pub mod ns;
pub mod transport;
pub mod error;
pub mod client;
pub mod plugin;
pub mod event;
pub mod plugins;
pub mod connection;
pub mod sasl;

mod locked_io;
