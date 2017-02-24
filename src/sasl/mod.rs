//! Provides the `SaslMechanism` trait and some implementations.

pub trait SaslMechanism {
    /// The name of the mechanism.
    fn name() -> &'static str;

    /// Provides initial payload of the SASL mechanism.
    fn initial(&mut self) -> Vec<u8> {
        Vec::new()
    }

    /// Creates a response to the SASL challenge.
    fn response(&mut self, _challenge: &[u8]) -> Vec<u8> {
        Vec::new()
    }
}

pub mod mechanisms;
