// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::{Element, IntoElements, ElementEmitter};

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Muc;

impl TryFrom<Element> for Muc {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Muc, Error> {
        if !elem.is("x", ns::MUC) {
            return Err(Error::ParseError("This is not an x element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in x element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in x element."));
        }
        Ok(Muc)
    }
}

impl Into<Element> for Muc {
    fn into(self) -> Element {
        Element::builder("x")
                .ns(ns::MUC)
                .build()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    // 100
    NonAnonymousRoom,

    // 101
    AffiliationChange,

    // 102
    ConfigShowsUnavailableMembers,

    // 103
    ConfigHidesUnavailableMembers,

    // 104
    ConfigNonPrivacyRelated,

    // 110
    SelfPresence,

    // 170
    ConfigRoomLoggingEnabled,

    // 171
    ConfigRoomLoggingDisabled,

    // 172
    ConfigRoomNonAnonymous,

    // 173
    ConfigRoomSemiAnonymous,

    // 201
    RoomHasBeenCreated,

    // 210
    AssignedNick,

    // 301
    Banned,

    // 303
    NewNick,

    // 307
    Kicked,

    // 321
    RemovalFromRoom,

    // 322
    ConfigMembersOnly,

    // 332
    ServiceShutdown,
}

impl TryFrom<Element> for Status {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Status, Error> {
        if !elem.is("status", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a status element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in status element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "code" {
                return Err(Error::ParseError("Unknown attribute in status element."));
            }
        }
        let code = get_attr!(elem, "code", required);

        Ok(match code {
             100 => Status::NonAnonymousRoom,
             101 => Status::AffiliationChange,
             102 => Status::ConfigShowsUnavailableMembers,
             103 => Status::ConfigHidesUnavailableMembers,
             104 => Status::ConfigNonPrivacyRelated,
             110 => Status::SelfPresence,
             170 => Status::ConfigRoomLoggingEnabled,
             171 => Status::ConfigRoomLoggingDisabled,
             172 => Status::ConfigRoomNonAnonymous,
             173 => Status::ConfigRoomSemiAnonymous,
             201 => Status::RoomHasBeenCreated,
             210 => Status::AssignedNick,
             301 => Status::Banned,
             303 => Status::NewNick,
             307 => Status::Kicked,
             321 => Status::RemovalFromRoom,
             322 => Status::ConfigMembersOnly,
             332 => Status::ServiceShutdown,
             _ => return Err(Error::ParseError("Invalid status code.")),
        })
    }
}

impl Into<Element> for Status {
    fn into(self) -> Element {
        Element::builder("status")
                .ns(ns::MUC_USER)
                .attr("code", match self {
                     Status::NonAnonymousRoom => 100,
                     Status::AffiliationChange => 101,
                     Status::ConfigShowsUnavailableMembers => 102,
                     Status::ConfigHidesUnavailableMembers => 103,
                     Status::ConfigNonPrivacyRelated => 104,
                     Status::SelfPresence => 110,
                     Status::ConfigRoomLoggingEnabled => 170,
                     Status::ConfigRoomLoggingDisabled => 171,
                     Status::ConfigRoomNonAnonymous => 172,
                     Status::ConfigRoomSemiAnonymous => 173,
                     Status::RoomHasBeenCreated => 201,
                     Status::AssignedNick => 210,
                     Status::Banned => 301,
                     Status::NewNick => 303,
                     Status::Kicked => 307,
                     Status::RemovalFromRoom => 321,
                     Status::ConfigMembersOnly => 322,
                     Status::ServiceShutdown => 332,
                })
                .build()
    }
}

impl IntoElements for Status {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone)]
pub struct MucUser {
    status: Vec<Status>,
}

impl TryFrom<Element> for MucUser {
    type Error = Error;

    fn try_from(elem: Element) -> Result<MucUser, Error> {
        if !elem.is("x", ns::MUC_USER) {
            return Err(Error::ParseError("This is not an x element."));
        }
        let mut status = vec!();
        for child in elem.children() {
            if child.is("status", ns::MUC_USER) {
                status.push(Status::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in x element."));
            }
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in x element."));
        }
        Ok(MucUser {
            status: status,
        })
    }
}

impl Into<Element> for MucUser {
    fn into(self) -> Element {
        Element::builder("x")
                .ns(ns::MUC_USER)
                .append(self.status)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_muc_simple() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        Muc::try_from(elem).unwrap();
    }

    #[test]
    fn test_muc_invalid_child() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'><coucou/></x>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_muc_serialise() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc'/>".parse().unwrap();
        let muc = Muc;
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_muc_invalid_attribute() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc' coucou=''/>".parse().unwrap();
        let error = Muc::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }

    #[test]
    fn test_muc_user_simple() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc#user'/>".parse().unwrap();
        MucUser::try_from(elem).unwrap();
    }

    #[test]
    fn test_muc_user_invalid_child() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc#user'><coucou/></x>".parse().unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_muc_user_serialise() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc#user'/>".parse().unwrap();
        let muc = MucUser { status: vec!() };
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_muc_user_invalid_attribute() {
        let elem: Element = "<x xmlns='http://jabber.org/protocol/muc#user' coucou=''/>".parse().unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }

    #[test]
    fn test_status_simple() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user' code='110'/>".parse().unwrap();
        Status::try_from(elem).unwrap();
    }

    #[test]
    fn test_status_invalid() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user'/>".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'code' missing.");
    }

    #[test]
    fn test_status_invalid_child() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user' code='110'><foo/></status>".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    fn test_status_simple_code() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user' code='307'/>".parse().unwrap();
        let status = Status::try_from(elem).unwrap();
        assert_eq!(status, Status::Kicked);
    }

    #[test]
    fn test_status_invalid_code() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user' code='666'/>".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid status code.");
    }

    #[test]
    fn test_status_invalid_code2() {
        let elem: Element = "<status xmlns='http://jabber.org/protocol/muc#user' code='coucou'/>".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "invalid digit found in string");
    }
}
