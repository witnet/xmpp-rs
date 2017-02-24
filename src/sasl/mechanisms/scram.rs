//! Provides the SASL "SCRAM-*" mechanisms and a way to implement more.

use base64;

use sasl::SaslMechanism;

use error::Error;

use openssl::pkcs5::pbkdf2_hmac;
use openssl::hash::hash;
use openssl::hash::MessageDigest;
use openssl::sign::Signer;
use openssl::pkey::PKey;
use openssl::rand::rand_bytes;
use openssl::error::ErrorStack;

use std::marker::PhantomData;

#[cfg(test)]
#[test]
fn xor_works() {
    assert_eq!( xor( &[135, 94, 53, 134, 73, 233, 140, 221, 150, 12, 96, 111, 54, 66, 11, 76]
                   , &[163, 9, 122, 180, 107, 44, 22, 252, 248, 134, 112, 82, 84, 122, 56, 209] )
              , &[36, 87, 79, 50, 34, 197, 154, 33, 110, 138, 16, 61, 98, 56, 51, 157] );
}

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    let mut ret = Vec::with_capacity(a.len());
    for (a, b) in a.into_iter().zip(b) {
        ret.push(a ^ b);
    }
    ret
}

fn generate_nonce() -> Result<String, ErrorStack> {
    let mut data = vec![0; 32];
    rand_bytes(&mut data)?;
    Ok(base64::encode(&data))
}

pub trait ScramProvider {
    fn name() -> &'static str;
    fn hash(data: &[u8]) -> Vec<u8>;
    fn hmac(data: &[u8], key: &[u8]) -> Vec<u8>;
    fn derive(data: &[u8], salt: &[u8], iterations: usize) -> Vec<u8>;
}

pub struct Sha1;

impl ScramProvider for Sha1 { // TODO: look at all these unwraps
    fn name() -> &'static str { "SCRAM-SHA-1" }

    fn hash(data: &[u8]) -> Vec<u8> {
        hash(MessageDigest::sha1(), data).unwrap()
    }

    fn hmac(data: &[u8], key: &[u8]) -> Vec<u8> {
        let pkey = PKey::hmac(key).unwrap();
        let mut signer = Signer::new(MessageDigest::sha1(), &pkey).unwrap();
        signer.update(data).unwrap();
        signer.finish().unwrap()
    }

    fn derive(data: &[u8], salt: &[u8], iterations: usize) -> Vec<u8> {
        let mut result = vec![0; 20];
        pbkdf2_hmac(data, salt, iterations, MessageDigest::sha1(), &mut result).unwrap();
        result
    }
}

pub struct Sha256;

impl ScramProvider for Sha256 { // TODO: look at all these unwraps
    fn name() -> &'static str { "SCRAM-SHA-256" }

    fn hash(data: &[u8]) -> Vec<u8> {
        hash(MessageDigest::sha256(), data).unwrap()
    }

    fn hmac(data: &[u8], key: &[u8]) -> Vec<u8> {
        let pkey = PKey::hmac(key).unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &pkey).unwrap();
        signer.update(data).unwrap();
        signer.finish().unwrap()
    }

    fn derive(data: &[u8], salt: &[u8], iterations: usize) -> Vec<u8> {
        let mut result = vec![0; 32];
        pbkdf2_hmac(data, salt, iterations, MessageDigest::sha256(), &mut result).unwrap();
        result
    }
}

enum ScramState {
    Init,
    SentInitialMessage { initial_message: Vec<u8> },
    GotServerData { server_signature: Vec<u8> },
}

pub struct Scram<S: ScramProvider> {
    name: String,
    password: String,
    client_nonce: String,
    state: ScramState,
    _marker: PhantomData<S>,
}

impl<S: ScramProvider> Scram<S> {
    pub fn new<N: Into<String>, P: Into<String>>(name: N, password: P) -> Result<Scram<S>, Error> {
        Ok(Scram {
            name: name.into(),
            password: password.into(),
            client_nonce: generate_nonce()?,
            state: ScramState::Init,
            _marker: PhantomData,
        })
    }

    pub fn new_with_nonce<N: Into<String>, P: Into<String>>(name: N, password: P, nonce: String) -> Scram<S> {
        Scram {
            name: name.into(),
            password: password.into(),
            client_nonce: nonce,
            state: ScramState::Init,
            _marker: PhantomData,
        }
    }
}

impl<S: ScramProvider> SaslMechanism for Scram<S> {
    fn name() -> &'static str {
        S::name()
    }

    fn initial(&mut self) -> Result<Vec<u8>, String> {
        let mut bare = Vec::new();
        bare.extend(b"n=");
        bare.extend(self.name.bytes());
        bare.extend(b",r=");
        bare.extend(self.client_nonce.bytes());
        self.state = ScramState::SentInitialMessage { initial_message: bare.clone() };
        let mut data = Vec::new();
        data.extend(b"n,,");
        data.extend(bare);
        Ok(data)
    }

    fn response(&mut self, challenge: &[u8]) -> Result<Vec<u8>, String> {
        let next_state;
        let ret;
        match self.state {
            ScramState::SentInitialMessage { ref initial_message } => {
                let chal = String::from_utf8(challenge.to_owned()).map_err(|_| "can't decode challenge".to_owned())?;
                let mut server_nonce: Option<String> = None;
                let mut salt: Option<Vec<u8>> = None;
                let mut iterations: Option<usize> = None;
                for s in chal.split(',') {
                    let mut tmp = s.splitn(2, '=');
                    let key = tmp.next();
                    if let Some(val) = tmp.next() {
                        match key {
                            Some("r") => {
                                if val.starts_with(&self.client_nonce) {
                                    server_nonce = Some(val.to_owned());
                                }
                            },
                            Some("s") => {
                                if let Ok(s) = base64::decode(val) {
                                    salt = Some(s);
                                }
                            },
                            Some("i") => {
                                if let Ok(iters) = val.parse() {
                                    iterations = Some(iters);
                                }
                            },
                            _ => (),
                        }
                    }
                }
                let server_nonce = server_nonce.ok_or_else(|| "no server nonce".to_owned())?;
                let salt = salt.ok_or_else(|| "no server salt".to_owned())?;
                let iterations = iterations.ok_or_else(|| "no server iterations".to_owned())?;
                // TODO: SASLprep
                let mut client_final_message_bare = Vec::new();
                client_final_message_bare.extend(b"c=biws,r=");
                client_final_message_bare.extend(server_nonce.bytes());
                let salted_password = S::derive(self.password.as_bytes(), &salt, iterations);
                let client_key = S::hmac(b"Client Key", &salted_password);
                let server_key = S::hmac(b"Server Key", &salted_password);
                let mut auth_message = Vec::new();
                auth_message.extend(initial_message);
                auth_message.push(b',');
                auth_message.extend(challenge);
                auth_message.push(b',');
                auth_message.extend(&client_final_message_bare);
                let stored_key = S::hash(&client_key);
                let client_signature = S::hmac(&auth_message, &stored_key);
                let client_proof = xor(&client_key, &client_signature);
                let server_signature = S::hmac(&auth_message, &server_key);
                let mut client_final_message = Vec::new();
                client_final_message.extend(&client_final_message_bare);
                client_final_message.extend(b",p=");
                client_final_message.extend(base64::encode(&client_proof).bytes());
                next_state = ScramState::GotServerData {
                    server_signature: server_signature,
                };
                ret = client_final_message;
            },
            _ => { return Err("not in the right state to receive this response".to_owned()); }
        }
        self.state = next_state;
        Ok(ret)
    }

    fn success(&mut self, data: &[u8]) -> Result<(), String> {
        let data = String::from_utf8(data.to_owned()).map_err(|_| "can't decode success message".to_owned())?;
        let mut received_signature = None;
        match self.state {
            ScramState::GotServerData { ref server_signature } => {
                for s in data.split(',') {
                    let mut tmp = s.splitn(2, '=');
                    let key = tmp.next();
                    if let Some(val) = tmp.next() {
                        match key {
                            Some("v") => {
                                if let Ok(v) = base64::decode(val) {
                                    received_signature = Some(v);
                                }
                            },
                            _ => (),
                        }
                    }
                }
                if let Some(sig) = received_signature {
                    if sig == *server_signature {
                        Ok(())
                    }
                    else {
                        Err("invalid signature in success response".to_owned())
                    }
                }
                else {
                    Err("no signature in success response".to_owned())
                }
            },
            _ => Err("not in the right state to get a success response".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use sasl::SaslMechanism;

    use super::*;

    #[test]
    fn scram_sha1_works() { // Source: https://wiki.xmpp.org/web/SASLandSCRAM-SHA-1
        let username = "user";
        let password = "pencil";
        let client_nonce = "fyko+d2lbbFgONRv9qkxdawL";
        let client_init = b"n,,n=user,r=fyko+d2lbbFgONRv9qkxdawL";
        let server_init = b"r=fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j,s=QSXCR+Q6sek8bf92,i=4096";
        let client_final = b"c=biws,r=fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j,p=v0X8v3Bz2T0CJGbJQyF0X+HI4Ts=";
        let server_final = b"v=rmF9pqV8S7suAoZWja4dJRkFsKQ=";
        let mut mechanism = Scram::<Sha1>::new_with_nonce(username, password, client_nonce.to_owned());
        let init = mechanism.initial().unwrap();
        assert_eq!( String::from_utf8(init.clone()).unwrap()
                  , String::from_utf8(client_init[..].to_owned()).unwrap() ); // depends on ordering…
        let resp = mechanism.response(&server_init[..]).unwrap();
        assert_eq!( String::from_utf8(resp.clone()).unwrap()
                  , String::from_utf8(client_final[..].to_owned()).unwrap() ); // again, depends on ordering…
        mechanism.success(&server_final[..]).unwrap();
    }

    #[test]
    fn scram_sha256_works() { // Source: RFC 7677
        let username = "user";
        let password = "pencil";
        let client_nonce = "rOprNGfwEbeRWgbNEkqO";
        let client_init = b"n,,n=user,r=rOprNGfwEbeRWgbNEkqO";
        let server_init = b"r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let client_final = b"c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ=";
        let server_final = b"v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=";
        let mut mechanism = Scram::<Sha256>::new_with_nonce(username, password, client_nonce.to_owned());
        let init = mechanism.initial().unwrap();
        assert_eq!( String::from_utf8(init.clone()).unwrap()
                  , String::from_utf8(client_init[..].to_owned()).unwrap() ); // depends on ordering…
        let resp = mechanism.response(&server_init[..]).unwrap();
        assert_eq!( String::from_utf8(resp.clone()).unwrap()
                  , String::from_utf8(client_final[..].to_owned()).unwrap() ); // again, depends on ordering…
        mechanism.success(&server_final[..]).unwrap();
    }
}
