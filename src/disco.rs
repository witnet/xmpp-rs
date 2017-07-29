// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;
use ns;

use data_forms::{DataForm, DataFormType};

#[derive(Debug, Clone)]
pub struct DiscoInfoQuery {
    pub node: Option<String>,
}

impl TryFrom<Element> for DiscoInfoQuery {
    type Err = Error;

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

impl TryFrom<Element> for Feature {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Feature, Error> {
        if !elem.is("feature", ns::DISCO_INFO) {
            return Err(Error::ParseError("This is not a disco#info feature element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in disco#info feature element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "var" {
                return Err(Error::ParseError("Unknown attribute in disco#info feature element."));
            }
        }
        Ok(Feature {
            var: get_attr!(elem, "var", required)
        })
    }
}

impl From<Feature> for Element {
    fn from(feature: Feature) -> Element {
        Element::builder("feature")
                .ns(ns::DISCO_INFO)
                .attr("var", feature.var)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub category: String, // TODO: use an enum here.
    pub type_: String, // TODO: use an enum here.
    pub lang: Option<String>,
    pub name: Option<String>,
}

impl TryFrom<Element> for Identity {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Identity, Error> {
        if !elem.is("identity", ns::DISCO_INFO) {
            return Err(Error::ParseError("This is not a disco#info identity element."));
        }

        let category = get_attr!(elem, "category", required);
        if category == "" {
            return Err(Error::ParseError("Identity must have a non-empty 'category' attribute."))
        }

        let type_ = get_attr!(elem, "type", required);
        if type_ == "" {
            return Err(Error::ParseError("Identity must have a non-empty 'type' attribute."))
        }

        Ok(Identity {
            category: category,
            type_: type_,
            lang: get_attr!(elem, "xml:lang", optional),
            name: get_attr!(elem, "name", optional),
        })
    }
}

impl From<Identity> for Element {
    fn from(identity: Identity) -> Element {
        Element::builder("identity")
                .ns(ns::DISCO_INFO)
                .attr("category", identity.category)
                .attr("type", identity.type_)
                .attr("xml:lang", identity.lang)
                .attr("name", identity.name)
                .build()
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
    type Err = Error;

    fn try_from(elem: Element) -> Result<DiscoInfoResult, Error> {
        if !elem.is("query", ns::DISCO_INFO) {
            return Err(Error::ParseError("This is not a disco#info element."));
        }

        let mut result = DiscoInfoResult {
            node: get_attr!(elem, "node", optional),
            identities: vec!(),
            features: vec!(),
            extensions: vec!(),
        };

        for child in elem.children() {
            if child.is("feature", ns::DISCO_INFO) {
                let feature = Feature::try_from(child.clone())?;
                result.features.push(feature);
            } else if child.is("identity", ns::DISCO_INFO) {
                let identity = Identity::try_from(child.clone())?;
                result.identities.push(identity);
            } else if child.is("x", ns::DATA_FORMS) {
                let data_form = DataForm::try_from(child.clone())?;
                if data_form.type_ != DataFormType::Result_ {
                    return Err(Error::ParseError("Data form must have a 'result' type in disco#info."));
                }
                if data_form.form_type.is_none() {
                    return Err(Error::ParseError("Data form found without a FORM_TYPE."));
                }
                result.extensions.push(data_form);
            } else {
                return Err(Error::ParseError("Unknown element in disco#info."));
            }
        }

        if result.identities.is_empty() {
            return Err(Error::ParseError("There must be at least one identity in disco#info."));
        }
        if result.features.is_empty() {
            return Err(Error::ParseError("There must be at least one feature in disco#info."));
        }
        if !result.features.contains(&Feature { var: ns::DISCO_INFO.to_owned() }) {
            return Err(Error::ParseError("disco#info feature not present in disco#info."));
        }

        Ok(result)
    }
}

impl From<DiscoInfoResult> for Element {
    fn from(disco: DiscoInfoResult) -> Element {
        Element::builder("query")
                .ns(ns::DISCO_INFO)
                .attr("node", disco.node)
                .append(disco.identities)
                .append(disco.features)
                .append(disco.extensions)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct DiscoItemsQuery {
    pub node: Option<String>,
}

impl TryFrom<Element> for DiscoItemsQuery {
    type Err = Error;

    fn try_from(elem: Element) -> Result<DiscoItemsQuery, Error> {
        if !elem.is("query", ns::DISCO_ITEMS) {
            return Err(Error::ParseError("This is not a disco#items element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in disco#items."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "node" {
                return Err(Error::ParseError("Unknown attribute in disco#items."));
            }
        }
        Ok(DiscoItemsQuery {
            node: get_attr!(elem, "node", optional),
        })
    }
}

impl From<DiscoItemsQuery> for Element {
    fn from(disco: DiscoItemsQuery) -> Element {
        Element::builder("query")
                .ns(ns::DISCO_ITEMS)
                .attr("node", disco.node)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub jid: Jid,
    pub node: Option<String>,
    pub name: Option<String>,
}

impl TryFrom<Element> for Item {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Item, Error> {
        if !elem.is("item", ns::DISCO_ITEMS) {
            return Err(Error::ParseError("This is not an item element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in item element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "jid" && attr != "node" && attr != "name" {
                return Err(Error::ParseError("Unknown attribute in item element."));
            }
        }
        Ok(Item {
            jid: get_attr!(elem, "jid", required),
            node: get_attr!(elem, "node", optional),
            name: get_attr!(elem, "name", optional),
        })
    }
}

impl From<Item> for Element {
    fn from(item: Item) -> Element {
        Element::builder("item")
                .ns(ns::DISCO_ITEMS)
                .attr("jid", String::from(item.jid))
                .attr("node", item.node)
                .attr("name", item.name)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct DiscoItemsResult {
    pub node: Option<String>,
    pub items: Vec<Item>,
}

impl TryFrom<Element> for DiscoItemsResult {
    type Err = Error;

    fn try_from(elem: Element) -> Result<DiscoItemsResult, Error> {
        if !elem.is("query", ns::DISCO_ITEMS) {
            return Err(Error::ParseError("This is not a disco#items element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "node" {
                return Err(Error::ParseError("Unknown attribute in disco#items."));
            }
        }

        let mut items: Vec<Item> = vec!();
        for child in elem.children() {
            if child.is("item", ns::DISCO_ITEMS) {
                items.push(Item::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown element in disco#items."));
            }
        }

        Ok(DiscoItemsResult {
            node: get_attr!(elem, "node", optional),
            items: items,
        })
    }
}

impl From<DiscoItemsResult> for Element {
    fn from(disco: DiscoItemsResult) -> Element {
        Element::builder("query")
                .ns(ns::DISCO_ITEMS)
                .attr("node", disco.node)
                .append(disco.items)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
    fn test_extension() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>example</value></field></x></query>".parse().unwrap();
        let elem1 = elem.clone();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert_eq!(query.extensions.len(), 1);
        assert_eq!(query.extensions[0].form_type, Some(String::from("example")));

        let elem2 = query.into();
        assert_eq!(elem1, elem2);
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

    #[test]
    fn test_simple_items() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>".parse().unwrap();
        let query = DiscoItemsQuery::try_from(elem).unwrap();
        assert!(query.node.is_none());

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items' node='coucou'/>".parse().unwrap();
        let query = DiscoItemsQuery::try_from(elem).unwrap();
        assert_eq!(query.node, Some(String::from("coucou")));
    }

    #[test]
    fn test_simple_items_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>".parse().unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert!(query.items.is_empty());

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items' node='coucou'/>".parse().unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        assert_eq!(query.node, Some(String::from("coucou")));
        assert!(query.items.is_empty());
    }

    #[test]
    fn test_answers_items_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'><item jid='component'/><item jid='component2' node='test' name='A component'/></query>".parse().unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        assert_eq!(query.items.len(), 2);
        assert_eq!(query.items[0].jid, Jid::from_str("component").unwrap());
        assert_eq!(query.items[0].node, None);
        assert_eq!(query.items[0].name, None);
        assert_eq!(query.items[1].jid, Jid::from_str("component2").unwrap());
        assert_eq!(query.items[1].node, Some(String::from("test")));
        assert_eq!(query.items[1].name, Some(String::from("A component")));
    }
}
