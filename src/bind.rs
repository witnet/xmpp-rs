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

#[derive(Debug, Clone)]
pub struct Bind {
    pub resource: Option<String>,
    pub jid: Option<Jid>,
}

impl Bind {
    pub fn new(resource: Option<String>) -> Bind {
        Bind {
            resource,
            jid: None,
        }
    }
}

impl TryFrom<Element> for Bind {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Bind, Error> {
        check_self!(elem, "bind", ns::BIND);
        check_no_attributes!(elem, "bind");

        let mut bind = Bind {
            resource: None,
            jid: None,
        };
        let mut already_set = false;
        for child in elem.children() {
            if already_set {
                return Err(Error::ParseError("Bind can only have one child."));
            }
            if child.is("resource", ns::BIND) {
                check_no_children!(child, "resource");
                bind.resource = Some(child.text());
                already_set = true;
            } else if child.is("jid", ns::BIND) {
                check_no_children!(child, "jid");
                bind.jid = Some(Jid::from_str(&child.text())?);
                already_set = true;
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
                .append(bind.resource.map(|resource|
                     Element::builder("resource")
                             .ns(ns::BIND)
                             .append(resource)
                             .build()))
                .append(bind.jid.map(|jid|
                     Element::builder("jid")
                             .ns(ns::BIND)
                             .append(jid)
                             .build()))
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/>".parse().unwrap();
        let bind = Bind::try_from(elem).unwrap();
        assert_eq!(bind.resource, None);
        assert_eq!(bind.jid, None);
    }
}
