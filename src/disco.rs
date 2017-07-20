// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::{Element, IntoElements, ElementEmitter};

use error::Error;
use ns;

use data_forms::{DataForm, DataFormType};

#[derive(Debug, Clone)]
pub struct DiscoInfoQuery {
    pub node: Option<String>,
}

impl TryFrom<Element> for DiscoInfoQuery {
    type Error = Error;

    fn try_from(elem: Element) -> Result<DiscoInfoQuery, Error> {
        if !elem.is("query", ns::DISCO_INFO) {
            return Err(Error::ParseError("This is not a disco#info element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in disco#info."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "node" {
                return Err(Error::ParseError("Unknown attribute in disco#info."));
            }
        }
        Ok(DiscoInfoQuery {
            node: get_attr!(elem, "node", optional),
        })
    }
}

impl From<DiscoInfoQuery> for Element {
    fn from(disco: DiscoInfoQuery) -> Element {
        Element::builder("query")
                .ns(ns::DISCO_INFO)
                .attr("node", disco.node)
                .build()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub var: String,
}

impl Into<Element> for Feature {
    fn into(self) -> Element {
        Element::builder("feature")
                .ns(ns::DISCO_INFO)
                .attr("var", self.var)
                .build()
    }
}

impl IntoElements for Feature {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub category: String, // TODO: use an enum here.
    pub type_: String, // TODO: use an enum here.
    pub lang: Option<String>,
    pub name: Option<String>,
}

impl Into<Element> for Identity {
    fn into(self) -> Element {
        Element::builder("identity")
                .ns(ns::DISCO_INFO)
                .attr("category", self.category)
                .attr("type", self.type_)
                .attr("xml:lang", self.lang)
                .attr("name", self.name)
                .build()
    }
}

impl IntoElements for Identity {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone)]
pub struct DiscoInfoResult {
    pub node: Option<String>,
    pub identities: Vec<Identity>,
    pub features: Vec<Feature>,
    pub extensions: Vec<DataForm>,
}

impl TryFrom<Element> for DiscoInfoResult {
    type Error = Error;

    fn try_from(elem: Element) -> Result<DiscoInfoResult, Error> {
        if !elem.is("query", ns::DISCO_INFO) {
            return Err(Error::ParseError("This is not a disco#info element."));
        }

        let mut identities: Vec<Identity> = vec!();
        let mut features: Vec<Feature> = vec!();
        let mut extensions: Vec<DataForm> = vec!();

        let node = get_attr!(elem, "node", optional);

        for child in elem.children() {
            if child.is("feature", ns::DISCO_INFO) {
                let feature = get_attr!(child, "var", required);
                features.push(Feature {
                    var: feature,
                });
            } else if child.is("identity", ns::DISCO_INFO) {
                let category = get_attr!(child, "category", required);
                if category == "" {
                    return Err(Error::ParseError("Identity must have a non-empty 'category' attribute."))
                }

                let type_ = get_attr!(child, "type", required);
                if type_ == "" {
                    return Err(Error::ParseError("Identity must have a non-empty 'type' attribute."))
                }

                let lang = get_attr!(child, "xml:lang", optional);
                let name = get_attr!(child, "name", optional);
                identities.push(Identity {
                    category: category,
                    type_: type_,
                    lang: lang,
                    name: name,
                });
            } else if child.is("x", ns::DATA_FORMS) {
                let data_form = DataForm::try_from(child.clone())?;
                if data_form.type_ != DataFormType::Result_ {
                    return Err(Error::ParseError("Data form must have a 'result' type in disco#info."));
                }
                match data_form.form_type {
                    Some(_) => extensions.push(data_form),
                    None => return Err(Error::ParseError("Data form found without a FORM_TYPE.")),
                }
            } else {
                return Err(Error::ParseError("Unknown element in disco#info."));
            }
        }

        if identities.is_empty() {
            return Err(Error::ParseError("There must be at least one identity in disco#info."));
        }
        if features.is_empty() {
            return Err(Error::ParseError("There must be at least one feature in disco#info."));
        }
        if !features.contains(&Feature { var: ns::DISCO_INFO.to_owned() }) {
            return Err(Error::ParseError("disco#info feature not present in disco#info."));
        }

        Ok(DiscoInfoResult {
            node: node,
            identities: identities,
            features: features,
            extensions: extensions
        })
    }
}

impl Into<Element> for DiscoInfoResult {
    fn into(self) -> Element {
        for _ in self.extensions {
            panic!("Not yet implemented!");
        }
        Element::builder("query")
                .ns(ns::DISCO_INFO)
                .attr("node", self.node)
                .append(self.identities)
                .append(self.features)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert!(query.extensions.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><coucou/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown element in disco#info.");
    }

    #[test]
    fn test_invalid_identity() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'category' missing.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category=''/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a non-empty 'category' attribute.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou' type=''/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a non-empty 'type' attribute.");
    }

    #[test]
    fn test_invalid_feature() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><feature/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'var' missing.");
    }

    #[test]
    fn test_invalid_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'/>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "There must be at least one identity in disco#info.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "There must be at least one feature in disco#info.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#items'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "disco#info feature not present in disco#info.");
    }
}
