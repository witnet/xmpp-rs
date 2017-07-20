// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};
use jid::Jid;

use error::Error;
use ns;

type Group = String;

generate_attribute!(Subscription, "subscription", {
    None => "none",
    From => "from",
    To => "to",
    Both => "both",
    Remove => "remove",
});

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub jid: Jid,
    pub name: Option<String>,
    pub subscription: Option<Subscription>,
    pub groups: Vec<Group>,
}

impl TryFrom<Element> for Item {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Item, Error> {
        if !elem.is("item", ns::ROSTER) {
            return Err(Error::ParseError("This is not a roster item element."));
        }

        let mut item = Item {
            jid: get_attr!(elem, "jid", required),
            name: get_attr!(elem, "name", optional).and_then(|name| if name == "" { None } else { Some(name) }),
            subscription: get_attr!(elem, "subscription", optional),
            groups: vec!(),
        };
        for child in elem.children() {
            if !child.is("group", ns::ROSTER) {
                return Err(Error::ParseError("Unknown element in roster item element."));
            }
            for _ in child.children() {
                return Err(Error::ParseError("Roster item group can’t have children."));
            }
            item.groups.push(child.text());
        }
        Ok(item)
    }
}

impl Into<Element> for Item {
    fn into(self) -> Element {
        Element::builder("item")
                .ns(ns::ROSTER)
                .attr("jid", String::from(self.jid))
                .attr("name", self.name)
                .attr("subscription", self.subscription)
                .append(self.groups.iter().map(|group| Element::builder("group").ns(ns::ROSTER).append(group)).collect::<Vec<_>>())
                .build()
    }
}

impl IntoElements for Item {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone)]
pub struct Roster {
    pub ver: Option<String>,
    pub items: Vec<Item>,
}

impl TryFrom<Element> for Roster {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Roster, Error> {
        if !elem.is("query", ns::ROSTER) {
            return Err(Error::ParseError("This is not a roster element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "ver" {
                return Err(Error::ParseError("Unknown attribute in roster element."));
            }
        }

        let mut roster = Roster {
            ver: get_attr!(elem, "ver", optional),
            items: vec!(),
        };
        for child in elem.children() {
            if !child.is("item", ns::ROSTER) {
                return Err(Error::ParseError("Unknown element in roster element."));
            }
            let item = Item::try_from(child.clone())?;
            roster.items.push(item);
        }
        Ok(roster)
    }
}

impl Into<Element> for Roster {
    fn into(self) -> Element {
        Element::builder("query")
                .ns(ns::ROSTER)
                .attr("ver", self.ver)
                .append(self.items)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(roster.items[0].subscription, Some(Subscription::Both));
        assert_eq!(roster.items[0].groups, vec!(String::from("Friends")));
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
        assert_eq!(roster.items[0].groups[0], String::from("A"));
        assert_eq!(roster.items[0].groups[1], String::from("B"));
        let elem2 = roster.into();
        assert_eq!(elem1, elem2);
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
        assert_eq!(roster.items[0].groups[0], String::from("Servants"));

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
        assert_eq!(roster.items[0].subscription, Some(Subscription::Remove));
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><coucou/></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown element in roster element.");

        let elem: Element = "<query xmlns='jabber:iq:roster' coucou=''/>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in roster element.");
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
        assert_eq!(message, "Unknown element in roster item element.");
    }
}
