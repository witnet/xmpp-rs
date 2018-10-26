// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::str::FromStr;
use try_from::TryFrom;

use minidom::Element;

use error::Error;
use jid::Jid;
use ns;
use iq::{IqSetPayload, IqResultPayload};

/// The request for resource binding, which is the process by which a client
/// can obtain a full JID and start exchanging on the XMPP network.
///
/// See https://xmpp.org/rfcs/rfc6120.html#bind
#[derive(Debug, Clone, PartialEq)]
pub enum Bind {
    /// Requests no particular resource, a random one will be affected by the
    /// server.
    None,

    /// Requests this resource, the server may associate another one though.
    Resource(String),

    /// The full JID returned by the server for this client.
    Jid(Jid),
}

impl Bind {
    /// Creates a resource binding request.
    pub fn new(resource: Option<String>) -> Bind {
        match resource {
            None => Bind::None,
            Some(resource) => Bind::Resource(resource),
        }
    }
}

impl IqSetPayload for Bind {}
impl IqResultPayload for Bind {}

impl TryFrom<Element> for Bind {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Bind, Error> {
        check_self!(elem, "bind", BIND);
        check_no_attributes!(elem, "bind");

        let mut bind = Bind::None;
        for child in elem.children() {
            if bind != Bind::None {
                return Err(Error::ParseError("Bind can only have one child."));
            }
            if child.is("resource", ns::BIND) {
                check_no_children!(child, "resource");
                bind = Bind::Resource(child.text());
            } else if child.is("jid", ns::BIND) {
                check_no_children!(child, "jid");
                bind = Bind::Jid(Jid::from_str(&child.text())?);
            } else {
                return Err(Error::ParseError("Unknown element in bind."));
            }
        }

        Ok(bind)
    }
}

impl From<Bind> for Element {
    fn from(bind: Bind) -> Element {
        Element::builder("bind")
                .ns(ns::BIND)
                .append(match bind {
                     Bind::None => vec!(),
                     Bind::Resource(resource) => vec!(
                         Element::builder("resource")
                                 .ns(ns::BIND)
                                 .append(resource)
                                 .build()
                     ),
                     Bind::Jid(jid) => vec!(
                         Element::builder("jid")
                                 .ns(ns::BIND)
                                 .append(jid)
                                 .build()
                     ),
                 })
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        assert_size!(Bind, 80);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/>".parse().unwrap();
        let bind = Bind::try_from(elem).unwrap();
        assert_eq!(bind, Bind::None);
    }
}
