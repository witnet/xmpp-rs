//! Provides the SASL "PLAIN" mechanism.

use sasl::SaslMechanism;

pub struct Plain {
    username: String,
    password: String,
}

impl Plain {
    pub fn new<N: Into<String>, P: Into<String>>(username: N, password: P) -> Plain {
        Plain {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl SaslMechanism for Plain {
    fn name(&self) -> &str { "PLAIN" }

    fn initial(&mut self) -> Result<Vec<u8>, String> {
        let mut auth = Vec::new();
        auth.push(0);
        auth.extend(self.username.bytes());
        auth.push(0);
        auth.extend(self.password.bytes());
        Ok(auth)
    }
}
