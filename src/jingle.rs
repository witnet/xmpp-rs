// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::Element;
use jid::Jid;

use error::Error;
use ns;
use iq::IqSetPayload;

generate_attribute!(Action, "action", {
    ContentAccept => "content-accept",
    ContentAdd => "content-add",
    ContentModify => "content-modify",
    ContentReject => "content-reject",
    ContentRemove => "content-remove",
    DescriptionInfo => "description-info",
    SecurityInfo => "security-info",
    SessionAccept => "session-accept",
    SessionInfo => "session-info",
    SessionInitiate => "session-initiate",
    SessionTerminate => "session-terminate",
    TransportAccept => "transport-accept",
    TransportInfo => "transport-info",
    TransportReject => "transport-reject",
    TransportReplace => "transport-replace",
});

generate_attribute!(Creator, "creator", {
    Initiator => "initiator",
    Responder => "responder",
});

generate_attribute!(Senders, "senders", {
    Both => "both",
    Initiator => "initiator",
    None => "none",
    Responder => "responder",
}, Default = Both);

// From https://www.iana.org/assignments/cont-disp/cont-disp.xhtml
generate_attribute!(Disposition, "disposition", {
    Inline => "inline",
    Attachment => "attachment",
    FormData => "form-data",
    Signal => "signal",
    Alert => "alert",
    Icon => "icon",
    Render => "render",
    RecipientListHistory => "recipient-list-history",
    Session => "session",
    Aib => "aib",
    EarlySession => "early-session",
    RecipientList => "recipient-list",
    Notification => "notification",
    ByReference => "by-reference",
    InfoPackage => "info-package",
    RecordingSession => "recording-session",
}, Default = Session);

generate_id!(ContentId);

#[derive(Debug, Clone)]
pub struct Content {
    pub creator: Creator,
    pub disposition: Disposition,
    pub name: ContentId,
    pub senders: Senders,
    pub description: Option<Element>,
    pub transport: Option<Element>,
    pub security: Option<Element>,
}

impl Content {
    pub fn new(creator: Creator, name: ContentId) -> Content {
        Content {
            creator,
            name,
            disposition: Disposition::Session,
            senders: Senders::Both,
            description: None,
            transport: None,
            security: None,
        }
    }

    pub fn with_disposition(mut self, disposition: Disposition) -> Content {
        self.disposition = disposition;
        self
    }

    pub fn with_senders(mut self, senders: Senders) -> Content {
        self.senders = senders;
        self
    }

    pub fn with_description(mut self, description: Element) -> Content {
        self.description = Some(description);
        self
    }

    pub fn with_transport(mut self, transport: Element) -> Content {
        self.transport = Some(transport);
        self
    }

    pub fn with_security(mut self, security: Element) -> Content {
        self.security = Some(security);
        self
    }
}

impl TryFrom<Element> for Content {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Content, Error> {
        check_self!(elem, "content", JINGLE);
        check_no_unknown_attributes!(elem, "content", ["creator", "disposition", "name", "senders"]);

        let mut content = Content {
            creator: get_attr!(elem, "creator", required),
            disposition: get_attr!(elem, "disposition", default),
            name: get_attr!(elem, "name", required),
            senders: get_attr!(elem, "senders", default),
            description: None,
            transport: None,
            security: None,
        };
        for child in elem.children() {
            if child.name() == "description" {
                if content.description.is_some() {
                    return Err(Error::ParseError("Content must not have more than one description."));
                }
                content.description = Some(child.clone());
            } else if child.name() == "transport" {
                if content.transport.is_some() {
                    return Err(Error::ParseError("Content must not have more than one transport."));
                }
                content.transport = Some(child.clone());
            } else if child.name() == "security" {
                if content.security.is_some() {
                    return Err(Error::ParseError("Content must not have more than one security."));
                }
                content.security = Some(child.clone());
            } else {
                return Err(Error::ParseError("Unknown child in content element."));
            }
        }
        Ok(content)
    }
}

impl From<Content> for Element {
    fn from(content: Content) -> Element {
        Element::builder("content")
                .ns(ns::JINGLE)
                .attr("creator", content.creator)
                .attr("disposition", content.disposition)
                .attr("name", content.name)
                .attr("senders", content.senders)
                .append(content.description)
                .append(content.transport)
                .append(content.security)
                .build()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    AlternativeSession, //(String),
    Busy,
    Cancel,
    ConnectivityError,
    Decline,
    Expired,
    FailedApplication,
    FailedTransport,
    GeneralError,
    Gone,
    IncompatibleParameters,
    MediaError,
    SecurityError,
    Success,
    Timeout,
    UnsupportedApplications,
    UnsupportedTransports,
}

impl FromStr for Reason {
    type Err = Error;

    fn from_str(s: &str) -> Result<Reason, Error> {
        Ok(match s {
            "alternative-session" => Reason::AlternativeSession,
            "busy" => Reason::Busy,
            "cancel" => Reason::Cancel,
            "connectivity-error" => Reason::ConnectivityError,
            "decline" => Reason::Decline,
            "expired" => Reason::Expired,
            "failed-application" => Reason::FailedApplication,
            "failed-transport" => Reason::FailedTransport,
            "general-error" => Reason::GeneralError,
            "gone" => Reason::Gone,
            "incompatible-parameters" => Reason::IncompatibleParameters,
            "media-error" => Reason::MediaError,
            "security-error" => Reason::SecurityError,
            "success" => Reason::Success,
            "timeout" => Reason::Timeout,
            "unsupported-applications" => Reason::UnsupportedApplications,
            "unsupported-transports" => Reason::UnsupportedTransports,

            _ => return Err(Error::ParseError("Unknown reason.")),
        })
    }
}

impl From<Reason> for Element {
    fn from(reason: Reason) -> Element {
        Element::builder(match reason {
            Reason::AlternativeSession => "alternative-session",
            Reason::Busy => "busy",
            Reason::Cancel => "cancel",
            Reason::ConnectivityError => "connectivity-error",
            Reason::Decline => "decline",
            Reason::Expired => "expired",
            Reason::FailedApplication => "failed-application",
            Reason::FailedTransport => "failed-transport",
            Reason::GeneralError => "general-error",
            Reason::Gone => "gone",
            Reason::IncompatibleParameters => "incompatible-parameters",
            Reason::MediaError => "media-error",
            Reason::SecurityError => "security-error",
            Reason::Success => "success",
            Reason::Timeout => "timeout",
            Reason::UnsupportedApplications => "unsupported-applications",
            Reason::UnsupportedTransports => "unsupported-transports",
        }).build()
    }
}

#[derive(Debug, Clone)]
pub struct ReasonElement {
    pub reason: Reason,
    pub text: Option<String>,
}

impl TryFrom<Element> for ReasonElement {
    type Err = Error;

    fn try_from(elem: Element) -> Result<ReasonElement, Error> {
        check_self!(elem, "reason", JINGLE);
        let mut reason = None;
        let mut text = None;
        for child in elem.children() {
            if !child.has_ns(ns::JINGLE) {
                return Err(Error::ParseError("Reason contains a foreign element."));
            }
            match child.name() {
                "text" => {
                    if text.is_some() {
                        return Err(Error::ParseError("Reason must not have more than one text."));
                    }
                    text = Some(child.text());
                },
                name => {
                    if reason.is_some() {
                        return Err(Error::ParseError("Reason must not have more than one reason."));
                    }
                    reason = Some(name.parse()?);
                },
            }
        }
        let reason = reason.ok_or(Error::ParseError("Reason doesn’t contain a valid reason."))?;
        Ok(ReasonElement {
            reason: reason,
            text: text,
        })
    }
}

impl From<ReasonElement> for Element {
    fn from(reason: ReasonElement) -> Element {
        Element::builder("reason")
                .append(Element::from(reason.reason))
                .append(reason.text)
                .build()
    }
}

generate_id!(SessionId);

#[derive(Debug, Clone)]
pub struct Jingle {
    pub action: Action,
    pub initiator: Option<Jid>,
    pub responder: Option<Jid>,
    pub sid: SessionId,
    pub contents: Vec<Content>,
    pub reason: Option<ReasonElement>,
    pub other: Vec<Element>,
}

impl IqSetPayload for Jingle {}

impl Jingle {
    pub fn new(action: Action, sid: SessionId) -> Jingle {
        Jingle {
            action: action,
            sid: sid,
            initiator: None,
            responder: None,
            contents: Vec::new(),
            reason: None,
            other: Vec::new(),
        }
    }

    pub fn with_initiator(mut self, initiator: Jid) -> Jingle {
        self.initiator = Some(initiator);
        self
    }

    pub fn with_responder(mut self, responder: Jid) -> Jingle {
        self.responder = Some(responder);
        self
    }

    pub fn add_content(mut self, content: Content) -> Jingle {
        self.contents.push(content);
        self
    }

    pub fn set_reason(mut self, content: Content) -> Jingle {
        self.contents.push(content);
        self
    }
}

impl TryFrom<Element> for Jingle {
    type Err = Error;

    fn try_from(root: Element) -> Result<Jingle, Error> {
        check_self!(root, "jingle", JINGLE, "Jingle");
        check_no_unknown_attributes!(root, "Jingle", ["action", "initiator", "responder", "sid"]);

        let mut jingle = Jingle {
            action: get_attr!(root, "action", required),
            initiator: get_attr!(root, "initiator", optional),
            responder: get_attr!(root, "responder", optional),
            sid: get_attr!(root, "sid", required),
            contents: vec!(),
            reason: None,
            other: vec!(),
        };

        for child in root.children().cloned() {
            if child.is("content", ns::JINGLE) {
                let content = Content::try_from(child)?;
                jingle.contents.push(content);
            } else if child.is("reason", ns::JINGLE) {
                if jingle.reason.is_some() {
                    return Err(Error::ParseError("Jingle must not have more than one reason."));
                }
                let reason = ReasonElement::try_from(child)?;
                jingle.reason = Some(reason);
            } else {
                jingle.other.push(child);
            }
        }

        Ok(jingle)
    }
}

impl From<Jingle> for Element {
    fn from(jingle: Jingle) -> Element {
        Element::builder("jingle")
                .ns(ns::JINGLE)
                .attr("action", jingle.action)
                .attr("initiator", jingle.initiator)
                .attr("responder", jingle.responder)
                .attr("sid", jingle.sid)
                .append(jingle.contents)
                .append(jingle.reason)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'/>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.action, Action::SessionInitiate);
        assert_eq!(jingle.sid, SessionId(String::from("coucou")));
    }

    #[test]
    fn test_invalid_jingle() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1'/>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'action' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-info'/>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='coucou' sid='coucou'/>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'action' attribute.");
    }

    #[test]
    fn test_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].creator, Creator::Initiator);
        assert_eq!(jingle.contents[0].name, ContentId(String::from("coucou")));
        assert_eq!(jingle.contents[0].senders, Senders::Both);
        assert_eq!(jingle.contents[0].disposition, Disposition::Session);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='both'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].senders, Senders::Both);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' disposition='early-session'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].disposition, Disposition::EarlySession);
    }

    #[test]
    fn test_invalid_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'creator' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'name' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='coucou' name='coucou'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'creator' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='coucou'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'senders' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders=''/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'senders' attribute.");
    }

    #[test]
    fn test_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/></reason></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, Reason::Success);
        assert_eq!(reason.text, None);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/><text>coucou</text></reason></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, Reason::Success);
        assert_eq!(reason.text, Some(String::from("coucou")));
    }

    #[test]
    fn test_invalid_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason doesn’t contain a valid reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a xmlns='http://www.w3.org/1999/xhtml'/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason contains a foreign element.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/></reason><reason/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Jingle must not have more than one reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/><text/><text/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason must not have more than one text.");
    }
}
