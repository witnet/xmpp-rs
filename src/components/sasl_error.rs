use ns;
use minidom::Element;
use util::FromElement;

#[derive(Clone, Debug)]
pub enum Condition {
    Aborted,
    AccountDisabled,
    CredentialsExpired,
    EncryptionRequired,
    IncorrectEncoding,
    InvalidAuthzid,
    InvalidMechanism,
    MalformedRequest,
    MechanismTooWeak,
    NotAuthorized,
    TemporaryAuthFailure,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct SaslError {
    condition: Condition,
    text: Option<String>,
}

impl FromElement for SaslError {
    type Err = ();

    fn from_element(element: &Element) -> Result<SaslError, ()> {
        if !element.is("failure", ns::SASL) {
            return Err(());
        }
        let mut err = SaslError {
            condition: Condition::Unknown,
            text: None,
        };
        if let Some(text) = element.get_child("text", ns::SASL) {
            let desc = text.text();
            err.text = Some(desc);
        }
        if element.has_child("aborted", ns::SASL) {
            err.condition = Condition::Aborted;
        }
        else if element.has_child("account-disabled", ns::SASL) {
            err.condition = Condition::AccountDisabled;
        }
        else if element.has_child("credentials-expired", ns::SASL) {
            err.condition = Condition::CredentialsExpired;
        }
        else if element.has_child("encryption-required", ns::SASL) {
            err.condition = Condition::EncryptionRequired;
        }
        else if element.has_child("incorrect-encoding", ns::SASL) {
            err.condition = Condition::IncorrectEncoding;
        }
        else if element.has_child("invalid-authzid", ns::SASL) {
            err.condition = Condition::InvalidAuthzid;
        }
        else if element.has_child("malformed-request", ns::SASL) {
            err.condition = Condition::MalformedRequest;
        }
        else if element.has_child("mechanism-too-weak", ns::SASL) {
            err.condition = Condition::MechanismTooWeak;
        }
        else if element.has_child("not-authorized", ns::SASL) {
            err.condition = Condition::NotAuthorized;
        }
        else if element.has_child("temporary-auth-failure", ns::SASL) {
            err.condition = Condition::TemporaryAuthFailure;
        }
        else {
            /* RFC 6120 section 6.5:
             *
             * However, because additional error conditions might be defined in
             * the future, if an entity receives a SASL error condition that it
             * does not understand then it MUST treat the unknown condition as
             * a generic authentication failure, i.e., as equivalent to
             * <not-authorized/> (Section 6.5.10). */
            err.condition = Condition::NotAuthorized;
        }
        Ok(err)
    }
}
