// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_attribute!(
    /// Whether a conference bookmark should be joined automatically.
    Autojoin,
    "autojoin",
    bool
);

generate_element!(
    /// A conference bookmark.
    Conference, "conference", BOOKMARKS2,
    attributes: [
        /// Whether a conference bookmark should be joined automatically.
        autojoin: Default<Autojoin> = "autojoin",

        /// A user-defined name for this conference.
        name: Option<String> = "name",
    ],
    children: [
        /// The nick the user will use to join this conference.
        nick: Option<String> = ("nick", BOOKMARKS2) => String,

        /// The password required to join this conference.
        password: Option<String> = ("password", BOOKMARKS2) => String
    ]
);

impl Conference {
    /// Create a new conference.
    pub fn new() -> Conference {
        Conference {
            autojoin: Autojoin::False,
            name: None,
            nick: None,
            password: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns;
    use crate::pubsub::event::PubSubEvent;
    use crate::pubsub::pubsub::Item as PubSubItem;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Conference, 40);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Conference, 80);
    }

    #[test]
    fn simple() {
        let elem: Element = "<conference xmlns='urn:xmpp:bookmarks:0'/>"
            .parse()
            .unwrap();
        let elem1 = elem.clone();
        let conference = Conference::try_from(elem).unwrap();
        assert_eq!(conference.autojoin, Autojoin::False);
        assert_eq!(conference.name, None);
        assert_eq!(conference.nick, None);
        assert_eq!(conference.password, None);

        let elem2 = Element::from(Conference::new());
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn complete() {
        let elem: Element = "<conference xmlns='urn:xmpp:bookmarks:0' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference>".parse().unwrap();
        let conference = Conference::try_from(elem).unwrap();
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");
    }

    #[test]
    fn wrapped() {
        let elem: Element = "<item xmlns='http://jabber.org/protocol/pubsub' id='test-muc@muc.localhost'><conference xmlns='urn:xmpp:bookmarks:0' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference></item>".parse().unwrap();
        let item = PubSubItem::try_from(elem).unwrap();
        let payload = item.payload.clone().unwrap();
        println!("FOO: payload: {:?}", payload);
        // let conference = Conference::try_from(payload).unwrap();
        let conference = Conference::try_from(payload);
        println!("FOO: conference: {:?}", conference);
        /*
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");

        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='urn:xmpp:bookmarks:0'><item xmlns='http://jabber.org/protocol/pubsub#event' id='test-muc@muc.localhost'><conference xmlns='urn:xmpp:bookmarks:0' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference></item></items></event>".parse().unwrap();
        let mut items = match PubSubEvent::try_from(elem) {
            Ok(PubSubEvent::PublishedItems { node, items }) => {
                assert_eq!(&node.0, ns::BOOKMARKS2);
                items
            }
            _ => panic!(),
        };
        assert_eq!(items.len(), 1);
        let item = items.pop().unwrap();
        let payload = item.payload.clone().unwrap();
        let conference = Conference::try_from(payload).unwrap();
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");
        */
    }
}
