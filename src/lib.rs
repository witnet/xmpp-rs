extern crate xml;
extern crate openssl;
extern crate minidom;
extern crate base64;

pub mod ns;
pub mod transport;
pub mod error;
pub mod jid;
pub mod client;
pub mod plugin;
pub mod event;
pub mod plugins;
pub mod connection;

mod locked_io;
