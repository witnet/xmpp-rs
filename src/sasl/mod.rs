//! Provides the `SaslMechanism` trait and some implementations.

pub trait SaslMechanism {
    /// The name of the mechanism.
    fn name(&self) -> &str;

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
