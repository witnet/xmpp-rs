use ns;
use minidom::Element;
use util::{FromElement, FromParentElement};
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    Auth,
    Cancel,
    Continue,
    Modify,
    Wait,
}

impl FromStr for ErrorType {
    type Err = ();

    fn from_str(s: &str) -> Result<ErrorType, ()> {
        Ok(match s {
            "auth" => ErrorType::Auth,
            "cancel" => ErrorType::Cancel,
            "continue" => ErrorType::Continue,
            "modify" => ErrorType::Modify,
            "wait" => ErrorType::Wait,
            _ => { return Err(()); },
        })
    }
}

#[derive(Clone, Debug)]
pub enum Condition {
    BadRequest,
    Conflict,
    FeatureNotImplemented,
    Forbidden,
    Gone(Option<String>),
    InternalServerError,
    ItemNotFound,
    JidMalformed,
    NotAcceptable,
    NotAllowed,
    NotAuthorized,
    PolicyViolation,
    RecipientUnavailable,
    Redirect(Option<String>),
    RegistrationRequired,
    RemoteServerNotFound,
    RemoteServerTimeout,
    ResourceConstraint,
    ServiceUnavailable,
    SubscriptionRequired,
    UndefinedCondition,
    UnexpectedRequest,
}

impl FromParentElement for Condition {
    type Err = ();

    fn from_parent_element(elem: &Element) -> Result<Condition, ()> {
        if elem.has_child("bad-request", ns::STANZAS) {
            Ok(Condition::BadRequest)
        }
        else if elem.has_child("conflict", ns::STANZAS) {
            Ok(Condition::Conflict)
        }
        else if elem.has_child("feature-not-implemented", ns::STANZAS) {
            Ok(Condition::FeatureNotImplemented)
        }
        else if elem.has_child("forbidden", ns::STANZAS) {
            Ok(Condition::Forbidden)
        }
        else if let Some(alt) = elem.get_child("gone", ns::STANZAS) {
            let text = alt.text();
            let inner = if text == "" { None } else { Some(text) };
            Ok(Condition::Gone(inner))
        }
        else if elem.has_child("internal-server-error", ns::STANZAS) {
            Ok(Condition::InternalServerError)
        }
        else if elem.has_child("item-not-found", ns::STANZAS) {
            Ok(Condition::ItemNotFound)
        }
        else if elem.has_child("jid-malformed", ns::STANZAS) {
            Ok(Condition::JidMalformed)
        }
        else if elem.has_child("not-acceptable", ns::STANZAS) {
            Ok(Condition::NotAcceptable)
        }
        else if elem.has_child("not-allowed", ns::STANZAS) {
            Ok(Condition::NotAllowed)
        }
        else if elem.has_child("not-authorized", ns::STANZAS) {
            Ok(Condition::NotAuthorized)
        }
        else if elem.has_child("policy-violation", ns::STANZAS) {
            Ok(Condition::PolicyViolation)
        }
        else if elem.has_child("recipient-unavailable", ns::STANZAS) {
            Ok(Condition::RecipientUnavailable)
        }
        else if let Some(alt) = elem.get_child("redirect", ns::STANZAS) {
            let text = alt.text();
            let inner = if text == "" { None } else { Some(text) };
            Ok(Condition::Redirect(inner))
        }
        else if elem.has_child("registration-required", ns::STANZAS) {
            Ok(Condition::RegistrationRequired)
        }
        else if elem.has_child("remote-server-not-found", ns::STANZAS) {
            Ok(Condition::RemoteServerNotFound)
        }
        else if elem.has_child("remote-server-timeout", ns::STANZAS) {
            Ok(Condition::RemoteServerTimeout)
        }
        else if elem.has_child("resource-constraint", ns::STANZAS) {
            Ok(Condition::ResourceConstraint)
        }
        else if elem.has_child("service-unavailable", ns::STANZAS) {
            Ok(Condition::ServiceUnavailable)
        }
        else if elem.has_child("subscription-required", ns::STANZAS) {
            Ok(Condition::SubscriptionRequired)
        }
        else if elem.has_child("undefined-condition", ns::STANZAS) {
            Ok(Condition::UndefinedCondition)
        }
        else if elem.has_child("unexpected-request", ns::STANZAS) {
            Ok(Condition::UnexpectedRequest)
        }
        else {
            Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct StanzaError {
    error_type: ErrorType,
    text: Option<String>,
    condition: Condition,
}

impl StanzaError {
    pub fn new(error_type: ErrorType, text: Option<String>, condition: Condition) -> StanzaError {
        StanzaError {
            error_type: error_type,
            text: text,
            condition: condition,
        }
    }
}

impl FromElement for StanzaError {
    type Err = ();

    fn from_element(elem: &Element) -> Result<StanzaError, ()> {
        if elem.is("error", ns::STANZAS) {
            let error_type = elem.attr("type").ok_or(())?;
            let err: ErrorType = error_type.parse().map_err(|_| ())?;
            let condition: Condition = Condition::from_parent_element(elem)?;
            let text = elem.get_child("text", ns::STANZAS).map(|c| c.text());
            Ok(StanzaError {
                error_type: err,
                text: text,
                condition: condition,
            })
        }
        else {
            Err(())
        }
    }
}
