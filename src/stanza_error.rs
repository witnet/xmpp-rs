// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;
use std::collections::BTreeMap;

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};

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

#[derive(Debug, Clone, PartialEq)]
pub enum DefinedCondition {
    BadRequest,
    Conflict,
    FeatureNotImplemented,
    Forbidden,
    Gone,
    InternalServerError,
    ItemNotFound,
    JidMalformed,
    NotAcceptable,
    NotAllowed,
    NotAuthorized,
    PolicyViolation,
    RecipientUnavailable,
    Redirect,
    RegistrationRequired,
    RemoteServerNotFound,
    RemoteServerTimeout,
    ResourceConstraint,
    ServiceUnavailable,
    SubscriptionRequired,
    UndefinedCondition,
    UnexpectedRequest,
}

impl FromStr for DefinedCondition {
    type Err = Error;

    fn from_str(s: &str) -> Result<DefinedCondition, Error> {
        Ok(match s {
            "bad-request" => DefinedCondition::BadRequest,
            "conflict" => DefinedCondition::Conflict,
            "feature-not-implemented" => DefinedCondition::FeatureNotImplemented,
            "forbidden" => DefinedCondition::Forbidden,
            "gone" => DefinedCondition::Gone,
            "internal-server-error" => DefinedCondition::InternalServerError,
            "item-not-found" => DefinedCondition::ItemNotFound,
            "jid-malformed" => DefinedCondition::JidMalformed,
            "not-acceptable" => DefinedCondition::NotAcceptable,
            "not-allowed" => DefinedCondition::NotAllowed,
            "not-authorized" => DefinedCondition::NotAuthorized,
            "policy-violation" => DefinedCondition::PolicyViolation,
            "recipient-unavailable" => DefinedCondition::RecipientUnavailable,
            "redirect" => DefinedCondition::Redirect,
            "registration-required" => DefinedCondition::RegistrationRequired,
            "remote-server-not-found" => DefinedCondition::RemoteServerNotFound,
            "remote-server-timeout" => DefinedCondition::RemoteServerTimeout,
            "resource-constraint" => DefinedCondition::ResourceConstraint,
            "service-unavailable" => DefinedCondition::ServiceUnavailable,
            "subscription-required" => DefinedCondition::SubscriptionRequired,
            "undefined-condition" => DefinedCondition::UndefinedCondition,
            "unexpected-request" => DefinedCondition::UnexpectedRequest,

            _ => return Err(Error::ParseError("Unknown defined-condition.")),
        })
    }
}

impl IntoElements for DefinedCondition {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(Element::builder(match self {
            DefinedCondition::BadRequest => "bad-request",
            DefinedCondition::Conflict => "conflict",
            DefinedCondition::FeatureNotImplemented => "feature-not-implemented",
            DefinedCondition::Forbidden => "forbidden",
            DefinedCondition::Gone => "gone",
            DefinedCondition::InternalServerError => "internal-server-error",
            DefinedCondition::ItemNotFound => "item-not-found",
            DefinedCondition::JidMalformed => "jid-malformed",
            DefinedCondition::NotAcceptable => "not-acceptable",
            DefinedCondition::NotAllowed => "not-allowed",
            DefinedCondition::NotAuthorized => "not-authorized",
            DefinedCondition::PolicyViolation => "policy-violation",
            DefinedCondition::RecipientUnavailable => "recipient-unavailable",
            DefinedCondition::Redirect => "redirect",
            DefinedCondition::RegistrationRequired => "registration-required",
            DefinedCondition::RemoteServerNotFound => "remote-server-not-found",
            DefinedCondition::RemoteServerTimeout => "remote-server-timeout",
            DefinedCondition::ResourceConstraint => "resource-constraint",
            DefinedCondition::ServiceUnavailable => "service-unavailable",
            DefinedCondition::SubscriptionRequired => "subscription-required",
            DefinedCondition::UndefinedCondition => "undefined-condition",
            DefinedCondition::UnexpectedRequest => "unexpected-request",
        }).ns(ns::XMPP_STANZAS).build());
    }
}

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
        if !elem.is("error", ns::JABBER_CLIENT) {
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
            } else if child.ns() == Some(ns::XMPP_STANZAS) {
                if defined_condition.is_some() {
                    return Err(Error::ParseError("Error must not have more than one defined-condition."));
                }
                for _ in child.children() {
                    return Err(Error::ParseError("Unknown element in defined-condition."));
                }
                let condition = DefinedCondition::from_str(child.name())?;
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

impl Into<Element> for StanzaError {
    fn into(self) -> Element {
        let mut root = Element::builder("error")
                               .ns(ns::JABBER_CLIENT)
                               .attr("type", self.type_)
                               .attr("by", self.by.and_then(|by| Some(String::from(by))))
                               .append(self.defined_condition)
                               .build();
        for (lang, text) in self.texts {
            let elem = Element::builder("text")
                               .ns(ns::XMPP_STANZAS)
                               .attr("xml:lang", lang)
                               .append(text)
                               .build();
            root.append_child(elem);
        }
        if let Some(other) = self.other {
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
        let elem: Element = "<error xmlns='jabber:client' type='cancel'><undefined-condition xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></error>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap();
        assert_eq!(error.type_, ErrorType::Cancel);
        assert_eq!(error.defined_condition, DefinedCondition::UndefinedCondition);
    }

    #[test]
    fn test_invalid_type() {
        let elem: Element = "<error xmlns='jabber:client'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<error xmlns='jabber:client' type='coucou'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'type' attribute.");
    }

    #[test]
    fn test_invalid_condition() {
        let elem: Element = "<error xmlns='jabber:client' type='cancel'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Error must have a defined-condition.");
    }
}
