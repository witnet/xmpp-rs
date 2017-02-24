//! Provides the SASL "ANONYMOUS" mechanism.

use sasl::SaslMechanism;

pub struct Anonymous;

impl Anonymous {
    pub fn new() -> Anonymous {
        Anonymous
    }
}

impl SaslMechanism for Anonymous {
    fn name() -> &'static str { "ANONYMOUS" }
}
