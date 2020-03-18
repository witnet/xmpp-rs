//! Contains wrapper for `<stream:features/>`

use xmpp_parsers::Element;
use crate::starttls::NS_XMPP_TLS;
use crate::client::{NS_XMPP_SASL, NS_XMPP_BIND};
use crate::error::AuthError;

/// Wraps `<stream:features/>`, usually the very first nonza of an
/// XMPPStream.
///
/// TODO: should this rather go into xmpp-parsers, kept in a decoded
/// struct?
pub struct StreamFeatures(pub Element);

impl StreamFeatures {
    /// Wrap the nonza
    pub fn new(element: Element) -> Self {
        StreamFeatures(element)
    }

    /// Can initiate TLS session with this server?
    pub fn can_starttls(&self) -> bool {
        self.0
            .get_child("starttls", NS_XMPP_TLS)
            .is_some()
    }

    /// Iterate over SASL mechanisms
    pub fn sasl_mechanisms<'a>(&'a self) -> Result<impl Iterator<Item=String> + 'a, AuthError> {
        Ok(self.0
           .get_child("mechanisms", NS_XMPP_SASL)
           .ok_or(AuthError::NoMechanism)?
           .children()
           .filter(|child| child.is("mechanism", NS_XMPP_SASL))
           .map(|mech_el| mech_el.text())
        )
    }

    /// Does server support user resource binding?
    pub fn can_bind(&self) -> bool {
        self.0
            .get_child("bind", NS_XMPP_BIND)
            .is_some()
    }
}
