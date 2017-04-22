extern crate minidom;

use std::str::FromStr;

use minidom::Element;

use error::Error;
use ns;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ContentAccept,
    ContentAdd,
    ContentModify,
    ContentReject,
    ContentRemove,
    DescriptionInfo,
    SecurityInfo,
    SessionAccept,
    SessionInfo,
    SessionInitiate,
    SessionTerminate,
    TransportAccept,
    TransportInfo,
    TransportReject,
    TransportReplace,
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Action, Error> {
        if s == "content-accept" {
            Ok(Action::ContentAccept)
        } else if s == "content-add" {
            Ok(Action::ContentAdd)
        } else if s == "content-modify" {
            Ok(Action::ContentModify)
        } else if s == "content-reject" {
            Ok(Action::ContentReject)
        } else if s == "content-remove" {
            Ok(Action::ContentRemove)
        } else if s == "description-info" {
            Ok(Action::DescriptionInfo)
        } else if s == "security-info" {
            Ok(Action::SecurityInfo)
        } else if s == "session-accept" {
            Ok(Action::SessionAccept)
        } else if s == "session-info" {
            Ok(Action::SessionInfo)
        } else if s == "session-initiate" {
            Ok(Action::SessionInitiate)
        } else if s == "session-terminate" {
            Ok(Action::SessionTerminate)
        } else if s == "transport-accept" {
            Ok(Action::TransportAccept)
        } else if s == "transport-info" {
            Ok(Action::TransportInfo)
        } else if s == "transport-reject" {
            Ok(Action::TransportReject)
        } else if s == "transport-replace" {
            Ok(Action::TransportReplace)
        } else {
            Err(Error::ParseError("Unknown action."))
        }
    }
}

// TODO: use a real JID type.
type Jid = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Creator {
    Initiator,
    Responder,
}

impl FromStr for Creator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Creator, Error> {
        if s == "initiator" {
            Ok(Creator::Initiator)
        } else if s == "responder" {
            Ok(Creator::Responder)
        } else {
            Err(Error::ParseError("Unknown creator."))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Senders {
    Both,
    Initiator,
    None_,
    Responder,
}

impl FromStr for Senders {
    type Err = Error;

    fn from_str(s: &str) -> Result<Senders, Error> {
        if s == "both" {
            Ok(Senders::Both)
        } else if s == "initiator" {
            Ok(Senders::Initiator)
        } else if s == "none" {
            Ok(Senders::None_)
        } else if s == "responder" {
            Ok(Senders::Responder)
        } else {
            Err(Error::ParseError("Unknown senders."))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Content {
    pub creator: Creator,
    pub disposition: String,
    pub name: String,
    pub senders: Senders,
    pub description: (String, Element),
    pub transport: (String, Element),
    pub security: Option<(String, Element)>,
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
        if s == "alternative-session" {
            Ok(Reason::AlternativeSession)
        } else if s == "busy" {
            Ok(Reason::Busy)
        } else if s == "cancel" {
            Ok(Reason::Cancel)
        } else if s == "connectivity-error" {
            Ok(Reason::ConnectivityError)
        } else if s == "decline" {
            Ok(Reason::Decline)
        } else if s == "expired" {
            Ok(Reason::Expired)
        } else if s == "failed-application" {
            Ok(Reason::FailedApplication)
        } else if s == "failed-transport" {
            Ok(Reason::FailedTransport)
        } else if s == "general-error" {
            Ok(Reason::GeneralError)
        } else if s == "gone" {
            Ok(Reason::Gone)
        } else if s == "incompatible-parameters" {
            Ok(Reason::IncompatibleParameters)
        } else if s == "media-error" {
            Ok(Reason::MediaError)
        } else if s == "security-error" {
            Ok(Reason::SecurityError)
        } else if s == "success" {
            Ok(Reason::Success)
        } else if s == "timeout" {
            Ok(Reason::Timeout)
        } else if s == "unsupported-applications" {
            Ok(Reason::UnsupportedApplications)
        } else if s == "unsupported-transports" {
            Ok(Reason::UnsupportedTransports)
        } else {
            Err(Error::ParseError("Unknown reason."))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasonElement {
    pub reason: Reason,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Jingle {
    pub action: Action,
    pub initiator: Option<Jid>,
    pub responder: Option<Jid>,
    pub sid: String,
    pub contents: Vec<Content>,
    pub reason: Option<ReasonElement>,
    //pub other: Vec<Element>,
}

pub fn parse_jingle(root: &Element) -> Result<Jingle, Error> {
    if !root.is("jingle", ns::JINGLE) {
        return Err(Error::ParseError("This is not a Jingle element."));
    }

    let mut contents: Vec<Content> = vec!();

    let action = root.attr("action")
                     .ok_or(Error::ParseError("Jingle must have an 'action' attribute."))?
                     .parse()?;
    let initiator = root.attr("initiator")
                        .and_then(|initiator| initiator.parse().ok());
    let responder = root.attr("responder")
                        .and_then(|responder| responder.parse().ok());
    let sid = root.attr("sid")
                  .ok_or(Error::ParseError("Jingle must have a 'sid' attribute."))?;
    let mut reason_element = None;

    for child in root.children() {
        if child.is("content", ns::JINGLE) {
            let creator = child.attr("creator")
                               .ok_or(Error::ParseError("Content must have a 'creator' attribute."))?
                               .parse()?;
            let disposition = child.attr("disposition")
                                   .unwrap_or("session");
            let name = child.attr("name")
                            .ok_or(Error::ParseError("Content must have a 'name' attribute."))?;
            let senders = child.attr("senders")
                               .unwrap_or("both")
                               .parse()?;
            let mut description = None;
            let mut transport = None;
            let mut security = None;
            for stuff in child.children() {
                if stuff.name() == "description" {
                    if description.is_some() {
                        return Err(Error::ParseError("Content must not have more than one description."));
                    }
                    let namespace = stuff.ns()
                                         .and_then(|ns| ns.parse().ok())
                                         // TODO: is this even reachable?
                                         .ok_or(Error::ParseError("Invalid namespace on description element."))?;
                    description = Some((
                        namespace,
                        stuff.clone(),
                    ));
                } else if stuff.name() == "transport" {
                    if transport.is_some() {
                        return Err(Error::ParseError("Content must not have more than one transport."));
                    }
                    let namespace = stuff.ns()
                                         .and_then(|ns| ns.parse().ok())
                                         // TODO: is this even reachable?
                                         .ok_or(Error::ParseError("Invalid namespace on transport element."))?;
                    transport = Some((
                        namespace,
                        stuff.clone(),
                    ));
                } else if stuff.name() == "security" {
                    if security.is_some() {
                        return Err(Error::ParseError("Content must not have more than one security."));
                    }
                    let namespace = stuff.ns()
                                         .and_then(|ns| ns.parse().ok())
                                         // TODO: is this even reachable?
                                         .ok_or(Error::ParseError("Invalid namespace on security element."))?;
                    security = Some((
                        namespace,
                        stuff.clone(),
                    ));
                }
            }
            if description.is_none() {
                return Err(Error::ParseError("Content must have one description."));
            }
            if transport.is_none() {
                return Err(Error::ParseError("Content must have one transport."));
            }
            let description = description.unwrap().to_owned();
            let transport = transport.unwrap().to_owned();
            contents.push(Content {
                creator: creator,
                disposition: disposition.to_owned(),
                name: name.to_owned(),
                senders: senders,
                description: description,
                transport: transport,
                security: security,
            });
        } else if child.is("reason", ns::JINGLE) {
            if reason_element.is_some() {
                return Err(Error::ParseError("Jingle must not have more than one reason."));
            }
            let mut reason = None;
            let mut text = None;
            for stuff in child.children() {
                if stuff.ns() != Some(ns::JINGLE) {
                    return Err(Error::ParseError("Reason contains a foreign element."));
                }
                let name = stuff.name();
                if name == "text" {
                    if text.is_some() {
                        return Err(Error::ParseError("Reason must not have more than one text."));
                    }
                    text = Some(stuff.text());
                } else {
                    reason = Some(name.parse()?);
                }
            }
            if reason.is_none() {
                return Err(Error::ParseError("Reason doesn’t contain a valid reason."));
            }
            reason_element = Some(ReasonElement {
                reason: reason.unwrap(),
                text: text,
            });
        } else {
            return Err(Error::ParseError("Unknown element in jingle."));
        }
    }

    Ok(Jingle {
        action: action,
        initiator: initiator,
        responder: responder,
        sid: sid.to_owned(),
        contents: contents,
        reason: reason_element,
    })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use jingle;

    #[test]
    fn test_simple() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'/>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        assert_eq!(jingle.action, jingle::Action::SessionInitiate);
        assert_eq!(jingle.sid, "coucou");
    }

    #[test]
    fn test_invalid_jingle() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1'/>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Jingle must have an 'action' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-info'/>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Jingle must have a 'sid' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='coucou' sid='coucou'/>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown action.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-accept' sid='coucou'><coucou/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown element in jingle.");
    }

    #[test]
    fn test_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        assert_eq!(jingle.contents[0].creator, jingle::Creator::Initiator);
        assert_eq!(jingle.contents[0].name, "coucou");
        assert_eq!(jingle.contents[0].senders, jingle::Senders::Both);
        assert_eq!(jingle.contents[0].disposition, "session");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='both'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        assert_eq!(jingle.contents[0].senders, jingle::Senders::Both);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' disposition='early-session'><description/><transport/></content></jingle>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        assert_eq!(jingle.contents[0].disposition, "early-session");
    }

    #[test]
    fn test_invalid_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Content must have a 'creator' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator'/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Content must have a 'name' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='coucou' name='coucou'/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown creator.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='coucou'/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown senders.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders=''/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown senders.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou'/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Content must have one description.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou'><description/></content></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Content must have one transport.");
    }

    #[test]
    fn test_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/></reason></jingle>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, jingle::Reason::Success);
        assert_eq!(reason.text, None);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/><text>coucou</text></reason></jingle>".parse().unwrap();
        let jingle = jingle::parse_jingle(&elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, jingle::Reason::Success);
        assert_eq!(reason.text, Some(String::from("coucou")));
    }

    #[test]
    fn test_invalid_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason doesn’t contain a valid reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a/></reason></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a xmlns='http://www.w3.org/1999/xhtml'/></reason></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason contains a foreign element.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/></reason><reason/></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Jingle must not have more than one reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/><text/><text/></reason></jingle>".parse().unwrap();
        let error = jingle::parse_jingle(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason must not have more than one text.");
    }
}
