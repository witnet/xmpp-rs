// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;
use std::collections::BTreeMap;

use minidom::{Element, IntoAttributeValue};

use error::Error;
use jid::Jid;
use ns;

generate_attribute!(ErrorType, "type", {
    Auth => "auth",
    Cancel => "cancel",
    Continue => "continue",
    Modify => "modify",
    Wait => "wait",
});

generate_element_enum!(DefinedCondition, "condition", ns::XMPP_STANZAS, {
    BadRequest => "bad-request",
    Conflict => "conflict",
    FeatureNotImplemented => "feature-not-implemented",
    Forbidden => "forbidden",
    Gone => "gone",
    InternalServerError => "internal-server-error",
    ItemNotFound => "item-not-found",
    JidMalformed => "jid-malformed",
    NotAcceptable => "not-acceptable",
    NotAllowed => "not-allowed",
    NotAuthorized => "not-authorized",
    PolicyViolation => "policy-violation",
    RecipientUnavailable => "recipient-unavailable",
    Redirect => "redirect",
    RegistrationRequired => "registration-required",
    RemoteServerNotFound => "remote-server-not-found",
    RemoteServerTimeout => "remote-server-timeout",
    ResourceConstraint => "resource-constraint",
    ServiceUnavailable => "service-unavailable",
    SubscriptionRequired => "subscription-required",
    UndefinedCondition => "undefined-condition",
    UnexpectedRequest => "unexpected-request",
});

pub type Lang = String;

#[derive(Debug, Clone)]
pub struct StanzaError {
    pub type_: ErrorType,
    pub by: Option<Jid>,
    pub defined_condition: DefinedCondition,
    pub texts: BTreeMap<Lang, String>,
    pub other: Option<Element>,
}

impl TryFrom<Element> for StanzaError {
    type Err = Error;

    fn try_from(elem: Element) -> Result<StanzaError, Error> {
        if !elem.is("error", ns::DEFAULT_NS) {
            return Err(Error::ParseError("This is not an error element."));
        }

        let type_ = get_attr!(elem, "type", required);
        let by = get_attr!(elem, "by", optional);
        let mut defined_condition = None;
        let mut texts = BTreeMap::new();
        let mut other = None;

        for child in elem.children() {
            if child.is("text", ns::XMPP_STANZAS) {
                for _ in child.children() {
                    return Err(Error::ParseError("Unknown element in error text."));
                }
                let lang = get_attr!(elem, "xml:lang", default);
                if texts.insert(lang, child.text()).is_some() {
                    return Err(Error::ParseError("Text element present twice for the same xml:lang."));
                }
            } else if child.has_ns(ns::XMPP_STANZAS) {
                if defined_condition.is_some() {
                    return Err(Error::ParseError("Error must not have more than one defined-condition."));
                }
                for _ in child.children() {
                    return Err(Error::ParseError("Unknown element in defined-condition."));
                }
                let condition = DefinedCondition::try_from(child.clone())?;
                defined_condition = Some(condition);
            } else {
                if other.is_some() {
                    return Err(Error::ParseError("Error must not have more than one other element."));
                }
                other = Some(child.clone());
            }
        }
        let defined_condition = defined_condition.ok_or(Error::ParseError("Error must have a defined-condition."))?;

        Ok(StanzaError {
            type_: type_,
            by: by,
            defined_condition: defined_condition,
            texts: texts,
            other: other,
        })
    }
}

impl From<StanzaError> for Element {
    fn from(err: StanzaError) -> Element {
        let mut root = Element::builder("error")
                               .ns(ns::DEFAULT_NS)
                               .attr("type", err.type_)
                               .attr("by", err.by)
                               .append(err.defined_condition)
                               .build();
        for (lang, text) in err.texts {
            let elem = Element::builder("text")
                               .ns(ns::XMPP_STANZAS)
                               .attr("xml:lang", lang)
                               .append(text)
                               .build();
            root.append_child(elem);
        }
        if let Some(other) = err.other {
            root.append_child(other);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='cancel'><undefined-condition xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></error>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='cancel'><undefined-condition xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></error>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap();
        assert_eq!(error.type_, ErrorType::Cancel);
        assert_eq!(error.defined_condition, DefinedCondition::UndefinedCondition);
    }

    #[test]
    fn test_invalid_type() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='coucou'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='coucou'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'type' attribute.");
    }

    #[test]
    fn test_invalid_condition() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='cancel'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='cancel'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Error must have a defined-condition.");
    }
}
