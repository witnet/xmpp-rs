// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;
use ns;

use data_forms::{DataForm, DataFormType, parse_data_form};

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub var: String,
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub category: String, // TODO: use an enum here.
    pub type_: String, // TODO: use an enum here.
    pub xml_lang: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Disco {
    pub node: Option<String>,
    pub identities: Vec<Identity>,
    pub features: Vec<Feature>,
    pub extensions: Vec<DataForm>,
}

pub fn parse_disco(root: &Element) -> Result<Disco, Error> {
    if !root.is("query", ns::DISCO_INFO) {
        return Err(Error::ParseError("This is not a disco#info element."));
    }

    let mut identities: Vec<Identity> = vec!();
    let mut features: Vec<Feature> = vec!();
    let mut extensions: Vec<DataForm> = vec!();

    let node = root.attr("node")
                   .and_then(|node| node.parse().ok());

    for child in root.children() {
        if child.is("feature", ns::DISCO_INFO) {
            let feature = child.attr("var")
                               .ok_or(Error::ParseError("Feature must have a 'var' attribute."))?;
            features.push(Feature {
                var: feature.to_owned(),
            });
        } else if child.is("identity", ns::DISCO_INFO) {
            let category = child.attr("category")
                                .ok_or(Error::ParseError("Identity must have a 'category' attribute."))?;
            if category == "" {
                return Err(Error::ParseError("Identity must have a non-empty 'category' attribute."))
            }

            let type_ = child.attr("type")
                             .ok_or(Error::ParseError("Identity must have a 'type' attribute."))?;
            if type_ == "" {
                return Err(Error::ParseError("Identity must have a non-empty 'type' attribute."))
            }

            let xml_lang = child.attr("xml:lang").unwrap_or("");
            let name = child.attr("name")
                            .and_then(|name| name.parse().ok());
            identities.push(Identity {
                category: category.to_owned(),
                type_: type_.to_owned(),
                xml_lang: xml_lang.to_owned(),
                name: name,
            });
        } else if child.is("x", ns::DATA_FORMS) {
            let data_form = parse_data_form(child)?;
            match data_form.type_ {
                DataFormType::Result_ => (),
                _ => return Err(Error::ParseError("Data form must have a 'result' type in disco#info.")),
            }
            match data_form.form_type {
                Some(_) => extensions.push(data_form),
                None => return Err(Error::ParseError("Data form found without a FORM_TYPE.")),
            }
        } else {
            return Err(Error::ParseError("Unknown element in disco#info."));
        }
    }

    /*
    // TODO: encode these restrictions only for result disco#info, not get ones.
    if identities.is_empty() {
        return Err(Error::ParseError("There must be at least one identity in disco#info."));
    }
    if features.is_empty() {
        return Err(Error::ParseError("There must be at least one feature in disco#info."));
    }
    if !features.contains(&Feature { var: ns::DISCO_INFO.to_owned() }) {
        return Err(Error::ParseError("disco#info feature not present in disco#info."));
    }
    */

    Ok(Disco {
        node: node,
        identities: identities,
        features: features,
        extensions: extensions
    })
}

pub fn serialise_disco(disco: &Disco) -> Element {
    let mut root = Element::builder("query")
                           .ns(ns::DISCO_INFO)
                           .attr("node", disco.node.clone())
                           .build();
    for identity in &disco.identities {
        let identity_element = Element::builder("identity")
                                       .ns(ns::DISCO_INFO)
                                       .attr("category", identity.category.clone())
                                       .attr("type", identity.type_.clone())
                                       .attr("xml:lang", identity.xml_lang.clone())
                                       .attr("name", identity.name.clone())
                                       .build();
        root.append_child(identity_element);
    }
    for feature in &disco.features {
        let feature_element = Element::builder("feature")
                                      .ns(ns::DISCO_INFO)
                                      .attr("var", feature.var.clone())
                                      .build();
        root.append_child(feature_element);
    }
    for _ in &disco.extensions {
        panic!("Not yet implemented!");
    }
    root
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use disco;

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let query = disco::parse_disco(&elem).unwrap();
        assert!(query.node.is_none());
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert!(query.extensions.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><coucou/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown element in disco#info.");
    }

    #[test]
    fn test_invalid_identity() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a 'category' attribute.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category=''/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a non-empty 'category' attribute.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou'/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a 'type' attribute.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou' type=''/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Identity must have a non-empty 'type' attribute.");
    }

    #[test]
    fn test_invalid_feature() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><feature/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Feature must have a 'var' attribute.");
    }

    #[test]
    #[ignore]
    fn test_invalid_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'/>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "There must be at least one identity in disco#info.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "There must be at least one feature in disco#info.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#items'/></query>".parse().unwrap();
        let error = disco::parse_disco(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "disco#info feature not present in disco#info.");
    }
}
