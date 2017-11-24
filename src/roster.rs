// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};
use jid::Jid;

use error::Error;
use ns;

generate_elem_id!(Group, "group", ns::ROSTER);

generate_attribute!(Subscription, "subscription", {
    None => "none",
    From => "from",
    To => "to",
    Both => "both",
    Remove => "remove",
}, Default = None);

generate_element_with_children!(
    /// Contact from the user’s contact list.
    #[derive(PartialEq)]
    Item, "item", ns::ROSTER,
    attributes: [
        /// JID of this contact.
        jid: Jid = "jid" => required,

        /// Name of this contact.
        name: Option<String> = "name" => optional_empty,

        /// Subscription status of this contact.
        subscription: Subscription = "subscription" => default
    ],

    children: [
        /// Groups this contact is part of.
        groups: Vec<Group> = ("group", ns::ROSTER) => Group
    ]
);

generate_element_with_children!(
    /// The contact list of the user.
    Roster, "query", ns::ROSTER,
    attributes: [
        /// Version of the contact list.
        ///
        /// This is an opaque string that should only be sent back to the server on
        /// a new connection, if this client is storing the contact list between
        /// connections.
        ver: Option<String> = "ver" => optional
    ],
    children: [
        /// List of the contacts of the user.
        items: Vec<Item> = ("item", ns::ROSTER) => Item
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_get() {
        let elem: Element = "<query xmlns='jabber:iq:roster'/>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert!(roster.items.is_empty());
    }

    #[test]
    fn test_result() {
        let elem: Element = "<query xmlns='jabber:iq:roster' ver='ver7'><item jid='nurse@example.com'/><item jid='romeo@example.net'/></query>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver7")));
        assert_eq!(roster.items.len(), 2);

        let elem2: Element = "<query xmlns='jabber:iq:roster' ver='ver7'><item jid='nurse@example.com'/><item jid='romeo@example.net' name=''/></query>".parse().unwrap();
        let roster2 = Roster::try_from(elem2).unwrap();
        assert_eq!(roster.items, roster2.items);

        let elem: Element = "<query xmlns='jabber:iq:roster' ver='ver9'/>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver9")));
        assert!(roster.items.is_empty());

        let elem: Element = r#"
<query xmlns='jabber:iq:roster' ver='ver11'>
  <item jid='romeo@example.net'
        name='Romeo'
        subscription='both'>
    <group>Friends</group>
  </item>
  <item jid='mercutio@example.com'
        name='Mercutio'
        subscription='from'/>
  <item jid='benvolio@example.net'
        name='Benvolio'
        subscription='both'/>
</query>
"#.parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver11")));
        assert_eq!(roster.items.len(), 3);
        assert_eq!(roster.items[0].jid, Jid::from_str("romeo@example.net").unwrap());
        assert_eq!(roster.items[0].name, Some(String::from("Romeo")));
        assert_eq!(roster.items[0].subscription, Subscription::Both);
        assert_eq!(roster.items[0].groups, vec!(Group::from_str("Friends").unwrap()));
    }

    #[test]
    fn test_multiple_groups() {
        let elem: Element = r#"
<query xmlns='jabber:iq:roster'>
  <item jid='test@example.org'>
    <group>A</group>
    <group>B</group>
  </item>
</query>
"#.parse().unwrap();
        let elem1 = elem.clone();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, Jid::from_str("test@example.org").unwrap());
        assert_eq!(roster.items[0].name, None);
        assert_eq!(roster.items[0].groups.len(), 2);
        assert_eq!(roster.items[0].groups[0], Group::from_str("A").unwrap());
        assert_eq!(roster.items[0].groups[1], Group::from_str("B").unwrap());
        let elem2 = roster.into();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn test_set() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><item jid='nurse@example.com'/></query>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);

        let elem: Element = r#"
<query xmlns='jabber:iq:roster'>
  <item jid='nurse@example.com'
        name='Nurse'>
    <group>Servants</group>
  </item>
</query>
"#.parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, Jid::from_str("nurse@example.com").unwrap());
        assert_eq!(roster.items[0].name, Some(String::from("Nurse")));
        assert_eq!(roster.items[0].groups.len(), 1);
        assert_eq!(roster.items[0].groups[0], Group::from_str("Servants").unwrap());

        let elem: Element = r#"
<query xmlns='jabber:iq:roster'>
  <item jid='nurse@example.com'
        subscription='remove'/>
</query>
"#.parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, Jid::from_str("nurse@example.com").unwrap());
        assert!(roster.items[0].name.is_none());
        assert!(roster.items[0].groups.is_empty());
        assert_eq!(roster.items[0].subscription, Subscription::Remove);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><coucou/></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in query element.");

        let elem: Element = "<query xmlns='jabber:iq:roster' coucou=''/>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in query element.");
    }

    #[test]
    fn test_invalid_item() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><item/></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'jid' missing.");

        /*
        let elem: Element = "<query xmlns='jabber:iq:roster'><item jid=''/></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let error = match error {
            Error::JidParseError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "Invalid JID, I guess?");
        */

        let elem: Element = "<query xmlns='jabber:iq:roster'><item jid='coucou'><coucou/></item></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in item element.");
    }
}
