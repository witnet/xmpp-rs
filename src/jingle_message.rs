// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;

use error::Error;

use jingle::SessionId;

use ns;

#[derive(Debug, Clone)]
pub enum JingleMI {
    Propose {
        sid: SessionId,
        // TODO: Use a more specialised type here.
        description: Element,
    },
    Retract(SessionId),
    Accept(SessionId),
    Proceed(SessionId),
    Reject(SessionId),
}

fn get_sid(elem: Element) -> Result<SessionId, Error> {
    for (attr, _) in elem.attrs() {
        if attr != "id" {
            return Err(Error::ParseError("Unknown attribute in Jingle message element."));
        }
    }
    Ok(SessionId(get_attr!(elem, "id", required)))
}

fn check_empty_and_get_sid(elem: Element) -> Result<SessionId, Error> {
    for _ in elem.children() {
        return Err(Error::ParseError("Unknown child in Jingle message element."));
    }
    get_sid(elem)
}

impl TryFrom<Element> for JingleMI {
    type Err = Error;

    fn try_from(elem: Element) -> Result<JingleMI, Error> {
        if !elem.has_ns(ns::JINGLE_MESSAGE) {
            return Err(Error::ParseError("This is not a Jingle message element."));
        }
        Ok(match elem.name() {
            "propose" => {
                let mut description = None;
                for child in elem.children() {
                    if child.name() != "description" {
                        return Err(Error::ParseError("Unknown child in propose element."));
                    }
                    if description.is_some() {
                        return Err(Error::ParseError("Too many children in propose element."));
                    }
                    description = Some(child.clone());
                }
                JingleMI::Propose {
                    sid: get_sid(elem)?,
                    description: description.ok_or(Error::ParseError("Propose element doesn’t contain a description."))?,
                }
            },
            "retract" => JingleMI::Retract(check_empty_and_get_sid(elem)?),
            "accept" => JingleMI::Accept(check_empty_and_get_sid(elem)?),
            "proceed" => JingleMI::Proceed(check_empty_and_get_sid(elem)?),
            "reject" => JingleMI::Reject(check_empty_and_get_sid(elem)?),
            _ => return Err(Error::ParseError("This is not a Jingle message element.")),
        })
    }
}

impl From<JingleMI> for Element {
    fn from(jingle_mi: JingleMI) -> Element {
        match jingle_mi {
            JingleMI::Propose { sid, description } => {
                Element::builder("propose")
                        .ns(ns::JINGLE_MESSAGE)
                        .attr("id", sid)
                        .append(description)
            },
            JingleMI::Retract(sid) => {
                Element::builder("retract")
                        .ns(ns::JINGLE_MESSAGE)
                        .attr("id", sid)
            }
            JingleMI::Accept(sid) => {
                Element::builder("accept")
                        .ns(ns::JINGLE_MESSAGE)
                        .attr("id", sid)
            }
            JingleMI::Proceed(sid) => {
                Element::builder("proceed")
                        .ns(ns::JINGLE_MESSAGE)
                        .attr("id", sid)
            }
            JingleMI::Reject(sid) => {
                Element::builder("reject")
                        .ns(ns::JINGLE_MESSAGE)
                        .attr("id", sid)
            }
        }.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<accept xmlns='urn:xmpp:jingle-message:0' id='coucou'/>".parse().unwrap();
        JingleMI::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<propose xmlns='urn:xmpp:jingle-message:0' id='coucou'><coucou/></propose>".parse().unwrap();
        let error = JingleMI::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in propose element.");
    }
}
