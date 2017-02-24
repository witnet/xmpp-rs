//! Provides the `SaslMechanism` trait and some implementations.

pub trait SaslMechanism {
    /// The name of the mechanism.
    fn name() -> &'static str;

    /// Provides initial payload of the SASL mechanism.
    fn initial(&mut self) -> Vec<u8> {
        Vec::new()
    }

    /// Creates a response to the SASL challenge.
    fn respond(&mut self, _challenge: &[u8]) -> Vec<u8> {
        Vec::new()
    }
}

/// A few SASL mechanisms.
pub mod mechanisms {
    use super::SaslMechanism;

    pub struct Anonymous;

    impl Anonymous {
        pub fn new() -> Anonymous {
            Anonymous
        }
    }

    impl SaslMechanism for Anonymous {
        fn name() -> &'static str { "ANONYMOUS" }
    }

    pub struct Plain {
        name: String,
        password: String,
    }

    impl Plain {
        pub fn new<N: Into<String>, P: Into<String>>(name: N, password: P) -> Plain {
            Plain {
                name: name.into(),
                password: password.into(),
            }
        }
    }

    impl SaslMechanism for Plain {
        fn name() -> &'static str { "PLAIN" }

        fn initial(&mut self) -> Vec<u8> {
            let mut auth = Vec::new();
            auth.push(0);
            auth.extend(self.name.bytes());
            auth.push(0);
            auth.extend(self.password.bytes());
            auth
        }
    }
}
