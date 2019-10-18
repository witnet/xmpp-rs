use xmpp_parsers::{Element, Jid};

/// High-level event on the Stream implemented by Client and Component
#[derive(Debug)]
pub enum Event {
    /// Stream is connected and initialized
    Online(Jid),
    /// Stream end
    Disconnected,
    /// Received stanza/nonza
    Stanza(Element),
}

impl Event {
    /// `Online` event?
    pub fn is_online(&self) -> bool {
        match *self {
            Event::Online(_) => true,
            _ => false,
        }
    }

    /// Get the server-assigned JID for the `Online` event
    pub fn get_jid(&self) -> Option<&Jid> {
        match *self {
            Event::Online(ref jid) => Some(jid),
            _ => None,
        }
    }

    /// `Stanza` event?
    pub fn is_stanza(&self, name: &str) -> bool {
        match *self {
            Event::Stanza(ref stanza) => stanza.name() == name,
            _ => false,
        }
    }

    /// If this is a `Stanza` event, get its data
    pub fn as_stanza(&self) -> Option<&Element> {
        match *self {
            Event::Stanza(ref stanza) => Some(stanza),
            _ => None,
        }
    }

    /// If this is a `Stanza` event, unwrap into its data
    pub fn into_stanza(self) -> Option<Element> {
        match self {
            Event::Stanza(stanza) => Some(stanza),
            _ => None,
        }
    }
}
