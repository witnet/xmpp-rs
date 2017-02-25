//! Provides the `SaslMechanism` trait and some implementations.

pub struct SaslCredentials {
    pub username: String,
    pub secret: SaslSecret,
    pub channel_binding: Option<Vec<u8>>,
}

pub enum SaslSecret {
    None,
    Password(String),
}

pub trait SaslMechanism {
    /// The name of the mechanism.
    fn name(&self) -> &str;

    /// Creates this mechanism from `SaslCredentials`.
    fn from_credentials(credentials: SaslCredentials) -> Result<Self, String> where Self: Sized;

    /// Provides initial payload of the SASL mechanism.
    fn initial(&mut self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    /// Creates a response to the SASL challenge.
    fn response(&mut self, _challenge: &[u8]) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    /// Verifies the server success response, if there is one.
    fn success(&mut self, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

pub mod mechanisms;
