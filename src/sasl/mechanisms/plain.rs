//! Provides the SASL "PLAIN" mechanism.

use sasl::SaslMechanism;

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

    fn initial(&mut self) -> Result<Vec<u8>, String> {
        let mut auth = Vec::new();
        auth.push(0);
        auth.extend(self.name.bytes());
        auth.push(0);
        auth.extend(self.password.bytes());
        Ok(auth)
    }
}
