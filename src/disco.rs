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

use iq::{IqGetPayload, IqResultPayload};
use data_forms::{DataForm, DataFormType};

generate_element!(
/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#info'/>` element.
///
/// It should only be used in an `<iq type='get'/>`, as it can only represent
/// the request, and not a result.
DiscoInfoQuery, "query", DISCO_INFO,
attributes: [
    /// Node on which we are doing the discovery.
    node: Option<String> = "node" => optional,
]);

impl IqGetPayload for DiscoInfoQuery {}

generate_element!(
/// Structure representing a `<feature xmlns='http://jabber.org/protocol/disco#info'/>` element.
#[derive(PartialEq)]
Feature, "feature", DISCO_INFO,
attributes: [
    /// Namespace of the feature we want to represent.
    var: String = "var" => required,
]);

/// Structure representing an `<identity xmlns='http://jabber.org/protocol/disco#info'/>` element.
#[derive(Debug, Clone)]
pub struct Identity {
    /// Category of this identity.
    pub category: String, // TODO: use an enum here.

    /// Type of this identity.
    pub type_: String, // TODO: use an enum here.

    /// Lang of the name of this identity.
    pub lang: Option<String>,

    /// Name of this identity.
    pub name: Option<String>,
}

impl TryFrom<Element> for Identity {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Identity, Error> {
        check_self!(elem, "identity", DISCO_INFO, "disco#info identity");
        check_no_children!(elem, "disco#info identity");
        check_no_unknown_attributes!(elem, "disco#info identity", ["category", "type", "xml:lang", "name"]);

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

/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#info'/>` element.
///
/// It should only be used in an `<iq type='result'/>`, as it can only
/// represent the result, and not a request.
#[derive(Debug, Clone)]
pub struct DiscoInfoResult {
    /// Node on which we have done this discovery.
    pub node: Option<String>,

    /// List of identities exposed by this entity.
    pub identities: Vec<Identity>,

    /// List of features supported by this entity.
    pub features: Vec<Feature>,

    /// List of extensions reported by this entity.
    pub extensions: Vec<DataForm>,
}

impl IqResultPayload for DiscoInfoResult {}

impl TryFrom<Element> for DiscoInfoResult {
    type Err = Error;

    fn try_from(elem: Element) -> Result<DiscoInfoResult, Error> {
        check_self!(elem, "query", DISCO_INFO, "disco#info result");
        check_no_unknown_attributes!(elem, "disco#info result", ["node"]);

        let mut result = DiscoInfoResult {
            node: get_attr!(elem, "node", optional),
            identities: vec!(),
            features: vec!(),
            extensions: vec!(),
        };

        for child in elem.children() {
            if child.is("identity", ns::DISCO_INFO) {
                let identity = Identity::try_from(child.clone())?;
                result.identities.push(identity);
            } else if child.is("feature", ns::DISCO_INFO) {
                let feature = Feature::try_from(child.clone())?;
                result.features.push(feature);
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

generate_element!(
/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#items'/>` element.
///
/// It should only be used in an `<iq type='get'/>`, as it can only represent
/// the request, and not a result.
DiscoItemsQuery, "query", DISCO_ITEMS,
attributes: [
    /// Node on which we are doing the discovery.
    node: Option<String> = "node" => optional,
]);

impl IqGetPayload for DiscoItemsQuery {}

generate_element!(
/// Structure representing an `<item xmlns='http://jabber.org/protocol/disco#items'/>` element.
Item, "item", DISCO_ITEMS,
attributes: [
    /// JID of the entity pointed by this item.
    jid: Jid = "jid" => required,
    /// Node of the entity pointed by this item.
    node: Option<String> = "node" => optional,
    /// Name of the entity pointed by this item.
    name: Option<String> = "name" => optional,
]);

generate_element!(
    /// Structure representing a `<query
    /// xmlns='http://jabber.org/protocol/disco#items'/>` element.
    ///
    /// It should only be used in an `<iq type='result'/>`, as it can only
    /// represent the result, and not a request.
    DiscoItemsResult, "query", DISCO_ITEMS,
    attributes: [
        /// Node on which we have done this discovery.
        node: Option<String> = "node" => optional
    ],
    children: [
        /// List of items pointed by this entity.
        items: Vec<Item> = ("item", DISCO_ITEMS) => Item
    ]
);

impl IqResultPayload for DiscoItemsResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use compare_elements::NamespaceAwareCompare;
    use std::str::FromStr;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Identity, 48);
        assert_size!(Feature, 12);
        assert_size!(DiscoInfoQuery, 12);
        assert_size!(DiscoInfoResult, 48);

        assert_size!(Item, 60);
        assert_size!(DiscoItemsQuery, 12);
        assert_size!(DiscoItemsResult, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Identity, 96);
        assert_size!(Feature, 24);
        assert_size!(DiscoInfoQuery, 24);
        assert_size!(DiscoInfoResult, 96);

        assert_size!(Item, 120);
        assert_size!(DiscoItemsQuery, 24);
        assert_size!(DiscoItemsResult, 48);
    }

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
    fn test_identity_after_feature() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><feature var='http://jabber.org/protocol/disco#info'/><identity category='client' type='pc'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert!(query.extensions.is_empty());
    }

    #[test]
    fn test_feature_after_dataform() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>coucou</value></field></x><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert_eq!(query.extensions.len(), 1);
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
        assert!(elem1.compare_to(&elem2));
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
        let elem2 = Element::from(query);
        let query = DiscoItemsResult::try_from(elem2).unwrap();
        assert_eq!(query.items.len(), 2);
        assert_eq!(query.items[0].jid, Jid::from_str("component").unwrap());
        assert_eq!(query.items[0].node, None);
        assert_eq!(query.items[0].name, None);
        assert_eq!(query.items[1].jid, Jid::from_str("component2").unwrap());
        assert_eq!(query.items[1].node, Some(String::from("test")));
        assert_eq!(query.items[1].name, Some(String::from("A component")));
    }
}
