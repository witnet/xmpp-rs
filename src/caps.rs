// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use disco::{Feature, Identity, DiscoInfoResult, DiscoInfoQuery};
use data_forms::DataForm;
use hashes::{Hash, Algo};

use minidom::Element;
use error::Error;
use ns;
use base64;

use sha_1::Sha1;
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};
use blake2::Blake2b;
use digest::{Digest, VariableOutput};

#[derive(Debug, Clone)]
pub struct Caps {
    pub ext: Option<String>,
    pub node: String,
    pub hash: Hash,
}

impl TryFrom<Element> for Caps {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Caps, Error> {
        if !elem.is("c", ns::CAPS) {
            return Err(Error::ParseError("This is not a caps element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in caps element."));
        }
        let hash = get_attr!(elem, "hash", required);
        let ver: String = get_attr!(elem, "ver", required);
        let hash = Hash {
            algo: hash,
            hash: base64::decode(&ver)?,
        };
        Ok(Caps {
            ext: get_attr!(elem, "ext", optional),
            node: get_attr!(elem, "node", required),
            hash: hash,
        })
    }
}

impl From<Caps> for Element {
    fn from(caps: Caps) -> Element {
        Element::builder("c")
                .ns(ns::CAPS)
                .attr("ext", caps.ext)
                .attr("hash", caps.hash.algo)
                .attr("node", caps.node)
                .attr("ver", base64::encode(&caps.hash.hash))
                .build()
    }
}

fn compute_item(field: &str) -> Vec<u8> {
    let mut bytes = field.as_bytes().to_vec();
    bytes.push(b'<');
    bytes
}

fn compute_items<T, F: Fn(&T) -> Vec<u8>>(things: &[T], encode: F) -> Vec<u8> {
    let mut string: Vec<u8> = vec!();
    let mut accumulator: Vec<Vec<u8>> = vec!();
    for thing in things {
        let bytes = encode(thing);
        accumulator.push(bytes);
    }
    // This works using the expected i;octet collation.
    accumulator.sort();
    for mut bytes in accumulator {
        string.append(&mut bytes);
    }
    string
}

fn compute_features(features: &[Feature]) -> Vec<u8> {
    compute_items(features, |feature| compute_item(&feature.var))
}

fn compute_identities(identities: &[Identity]) -> Vec<u8> {
    compute_items(identities, |identity| {
        let lang = identity.lang.clone().unwrap_or_default();
        let name = identity.name.clone().unwrap_or_default();
        let string = format!("{}/{}/{}/{}", identity.category, identity.type_, lang, name);
        let bytes = string.as_bytes();
        let mut vec = Vec::with_capacity(bytes.len());
        vec.extend_from_slice(bytes);
        vec.push(b'<');
        vec
    })
}

fn compute_extensions(extensions: &[DataForm]) -> Vec<u8> {
    compute_items(extensions, |extension| {
        let mut bytes = vec!();
        // TODO: maybe handle the error case?
        if let Some(ref form_type) = extension.form_type {
            bytes.extend_from_slice(form_type.as_bytes());
        }
        bytes.push(b'<');
        for field in extension.fields.clone() {
            if field.var == "FORM_TYPE" {
                continue;
            }
            bytes.append(&mut compute_item(&field.var));
            bytes.append(&mut compute_items(&field.values,
                                            |value| compute_item(value)));
        }
        bytes
    })
}

pub fn compute_disco(disco: &DiscoInfoResult) -> Vec<u8> {
    let identities_string = compute_identities(&disco.identities);
    let features_string = compute_features(&disco.features);
    let extensions_string = compute_extensions(&disco.extensions);

    let mut final_string = vec!();
    final_string.extend(identities_string);
    final_string.extend(features_string);
    final_string.extend(extensions_string);
    final_string
}

fn get_hash_vec(hash: &[u8]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(hash.len());
    vec.extend_from_slice(hash);
    vec
}

pub fn hash_caps(data: &[u8], algo: Algo) -> Result<Hash, String> {
    Ok(Hash {
        hash: match algo {
            Algo::Sha_1 => {
                let mut hasher = Sha1::default();
                hasher.input(data);
                let hash = hasher.result();
                get_hash_vec(hash.as_slice())
            },
            Algo::Sha_256 => {
                let mut hasher = Sha256::default();
                hasher.input(data);
                let hash = hasher.result();
                get_hash_vec(hash.as_slice())
            },
            Algo::Sha_512 => {
                let mut hasher = Sha512::default();
                hasher.input(data);
                let hash = hasher.result();
                get_hash_vec(hash.as_slice())
            },
            Algo::Sha3_256 => {
                let mut hasher = Sha3_256::default();
                hasher.input(data);
                let hash = hasher.result();
                get_hash_vec(hash.as_slice())
            },
            Algo::Sha3_512 => {
                let mut hasher = Sha3_512::default();
                hasher.input(data);
                let hash = hasher.result();
                get_hash_vec(hash.as_slice())
            },
            Algo::Blake2b_256 => {
                let mut hasher = Blake2b::default();
                hasher.input(data);
                let mut buf: [u8; 32] = [0; 32];
                let hash = hasher.variable_result(&mut buf).unwrap();
                get_hash_vec(hash)
            },
            Algo::Blake2b_512 => {
                let mut hasher = Blake2b::default();
                hasher.input(data);
                let mut buf: [u8; 64] = [0; 64];
                let hash = hasher.variable_result(&mut buf).unwrap();
                get_hash_vec(hash)
            },
            Algo::Unknown(algo) => return Err(format!("Unknown algorithm: {}.", algo)),
        },
        algo: algo,
    })
}

pub fn query_caps(caps: Caps) -> DiscoInfoQuery {
    DiscoInfoQuery {
        node: Some(format!("{}#{}", caps.node, base64::encode(&caps.hash.hash))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caps;
    use base64;

    #[test]
    fn test_parse() {
        let elem: Element = "<c xmlns='http://jabber.org/protocol/caps' hash='sha-256' node='coucou' ver='K1Njy3HZBThlo4moOD5gBGhn0U0oK7/CbfLlIUDi6o4='/>".parse().unwrap();
        let caps = Caps::try_from(elem).unwrap();
        assert_eq!(caps.node, String::from("coucou"));
        assert_eq!(caps.hash.algo, Algo::Sha_256);
        assert_eq!(caps.hash.hash, base64::decode("K1Njy3HZBThlo4moOD5gBGhn0U0oK7/CbfLlIUDi6o4=").unwrap());
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<c xmlns='http://jabber.org/protocol/caps'><hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>K1Njy3HZBThlo4moOD5gBGhn0U0oK7/CbfLlIUDi6o4=</hash></c>".parse().unwrap();
        let error = Caps::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in caps element.");
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let disco = DiscoInfoResult::try_from(elem).unwrap();
        let caps = caps::compute_disco(&disco);
        assert_eq!(caps.len(), 50);
    }

    #[test]
    fn test_xep_5_2() {
        let elem: Element = r#"
<query xmlns='http://jabber.org/protocol/disco#info'
       node='http://psi-im.org#q07IKJEyjvHSyhy//CH0CxmKi8w='>
  <identity category='client' name='Exodus 0.9.1' type='pc'/>
  <feature var='http://jabber.org/protocol/caps'/>
  <feature var='http://jabber.org/protocol/disco#info'/>
  <feature var='http://jabber.org/protocol/disco#items'/>
  <feature var='http://jabber.org/protocol/muc'/>
</query>
"#.parse().unwrap();
        
        let data = b"client/pc//Exodus 0.9.1<http://jabber.org/protocol/caps<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/disco#items<http://jabber.org/protocol/muc<";
        let mut expected = Vec::with_capacity(data.len());
        expected.extend_from_slice(data);
        let disco = DiscoInfoResult::try_from(elem).unwrap();
        let caps = caps::compute_disco(&disco);
        assert_eq!(caps, expected);

        let sha_1 = caps::hash_caps(&caps, Algo::Sha_1).unwrap();
        assert_eq!(sha_1.hash, base64::decode("QgayPKawpkPSDYmwT/WM94uAlu0=").unwrap());
    }

    #[test]
    fn test_xep_5_3() {
        let elem: Element = r#"
<query xmlns='http://jabber.org/protocol/disco#info'
       node='http://psi-im.org#q07IKJEyjvHSyhy//CH0CxmKi8w='>
  <identity xml:lang='en' category='client' name='Psi 0.11' type='pc'/>
  <identity xml:lang='el' category='client' name='Ψ 0.11' type='pc'/>
  <feature var='http://jabber.org/protocol/caps'/>
  <feature var='http://jabber.org/protocol/disco#info'/>
  <feature var='http://jabber.org/protocol/disco#items'/>
  <feature var='http://jabber.org/protocol/muc'/>
  <x xmlns='jabber:x:data' type='result'>
    <field var='FORM_TYPE' type='hidden'>
      <value>urn:xmpp:dataforms:softwareinfo</value>
    </field>
    <field var='ip_version'>
      <value>ipv4</value>
      <value>ipv6</value>
    </field>
    <field var='os'>
      <value>Mac</value>
    </field>
    <field var='os_version'>
      <value>10.5.1</value>
    </field>
    <field var='software'>
      <value>Psi</value>
    </field>
    <field var='software_version'>
      <value>0.11</value>
    </field>
  </x>
</query>
"#.parse().unwrap();
        let data = b"client/pc/el/\xce\xa8 0.11<client/pc/en/Psi 0.11<http://jabber.org/protocol/caps<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/disco#items<http://jabber.org/protocol/muc<urn:xmpp:dataforms:softwareinfo<ip_version<ipv4<ipv6<os<Mac<os_version<10.5.1<software<Psi<software_version<0.11<";
        let mut expected = Vec::with_capacity(data.len());
        expected.extend_from_slice(data);
        let disco = DiscoInfoResult::try_from(elem).unwrap();
        let caps = caps::compute_disco(&disco);
        assert_eq!(caps, expected);

        let sha_1 = caps::hash_caps(&caps, Algo::Sha_1).unwrap();
        assert_eq!(sha_1.hash, base64::decode("q07IKJEyjvHSyhy//CH0CxmKi8w=").unwrap());
    }
}
