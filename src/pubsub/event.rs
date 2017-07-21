// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};
use jid::Jid;
use chrono::{DateTime, FixedOffset};

use error::Error;

use ns;

use data_forms::DataForm;

#[derive(Debug, Clone)]
pub struct Item {
    payload: Option<Element>,
    id: Option<String>,
    node: Option<String>,
    publisher: Option<Jid>,
}

impl From<Item> for Element {
    fn from(item: Item) -> Element {
        Element::builder("item")
                .ns(ns::PUBSUB_EVENT)
                .attr("id", item.id)
                .attr("node", item.node)
                .attr("publisher", item.publisher.map(String::from))
                .append(item.payload)
                .build()
    }
}

generate_attribute!(Subscription, "subscription", {
    None => "none",
    Pending => "pending",
    Subscribed => "subscribed",
    Unconfigured => "unconfigured",
}, Default = None);

#[derive(Debug, Clone)]
pub enum PubSubEvent {
    /*
    Collection {
    },
    */
    Configuration {
        node: String,
        form: Option<DataForm>,
    },
    Delete {
        node: String,
        redirect: Option<String>,
    },
    EmptyItems {
        node: String,
    },
    PublishedItems {
        node: String,
        items: Vec<Item>,
    },
    RetractedItems {
        node: String,
        items: Vec<String>,
    },
    Purge {
        node: String,
    },
    Subscription {
        node: String,
        expiry: Option<DateTime<FixedOffset>>,
        jid: Option<Jid>,
        subid: Option<String>,
        subscription: Option<Subscription>,
    },
}

fn parse_items(elem: Element, node: String) -> Result<PubSubEvent, Error> {
    let mut is_retract = None;
    let mut items = vec!();
    let mut retracts = vec!();
    for child in elem.children() {
        if child.is("item", ns::PUBSUB_EVENT) {
            match is_retract {
                None => is_retract = Some(false),
                Some(false) => (),
                Some(true) => return Err(Error::ParseError("Mix of item and retract in items element.")),
            }
            let mut payloads = child.children().cloned().collect::<Vec<_>>();
            let payload = payloads.pop();
            if !payloads.is_empty() {
                return Err(Error::ParseError("More than a single payload in item element."));
            }
            let item = Item {
                payload,
                id: get_attr!(child, "id", optional),
                node: get_attr!(child, "node", optional),
                publisher: get_attr!(child, "publisher", optional),
            };
            items.push(item);
        } else if child.is("retract", ns::PUBSUB_EVENT) {
            match is_retract {
                None => is_retract = Some(true),
                Some(true) => (),
                Some(false) => return Err(Error::ParseError("Mix of item and retract in items element.")),
            }
            for _ in child.children() {
                return Err(Error::ParseError("Unknown child in retract element."));
            }
            for (attr, _) in child.attrs() {
                if attr != "id" {
                    return Err(Error::ParseError("Unknown attribute in retract element."));
                }
            }
            let id = get_attr!(child, "id", required);
            retracts.push(id);
        } else {
            return Err(Error::ParseError("Invalid child in items element."));
        }
    }
    Ok(match is_retract {
        None => PubSubEvent::EmptyItems { node },
        Some(false) => PubSubEvent::PublishedItems { node, items },
        Some(true) => PubSubEvent::RetractedItems { node, items: retracts },
    })
}

impl TryFrom<Element> for PubSubEvent {
    type Err = Error;

    fn try_from(elem: Element) -> Result<PubSubEvent, Error> {
        if !elem.is("event", ns::PUBSUB_EVENT) {
            return Err(Error::ParseError("This is not an event element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in event element."));
        }
        let mut payload = None;
        for child in elem.children() {
            /*
            for (attr, _) in child.attrs() {
                if attr != "node" {
                    return Err(Error::ParseError("Unknown attribute in items element."));
                }
            }
            */
            let node = get_attr!(child, "node", required);
            if child.is("configuration", ns::PUBSUB_EVENT) {
                let mut payloads = child.children().cloned().collect::<Vec<_>>();
                let item = payloads.pop();
                if !payloads.is_empty() {
                    return Err(Error::ParseError("More than a single payload in configuration element."));
                }
                let form = match item {
                    None => None,
                    Some(payload) => Some(DataForm::try_from(payload)?),
                };
                payload = Some(PubSubEvent::Configuration { node, form });
            } else if child.is("delete", ns::PUBSUB_EVENT) {
                let mut redirect = None;
                for item in child.children() {
                    if item.is("redirect", ns::PUBSUB_EVENT) {
                        if redirect.is_some() {
                            return Err(Error::ParseError("More than one redirect in delete element."));
                        }
                        let uri = get_attr!(item, "uri", required);
                        redirect = Some(uri);
                    } else {
                        return Err(Error::ParseError("Unknown child in delete element."));
                    }
                }
                payload = Some(PubSubEvent::Delete { node, redirect });
            } else if child.is("items", ns::PUBSUB_EVENT) {
                payload = Some(parse_items(child.clone(), node)?);
            } else if child.is("purge", ns::PUBSUB_EVENT) {
                for _ in child.children() {
                    return Err(Error::ParseError("Unknown child in purge element."));
                }
                payload = Some(PubSubEvent::Purge { node });
            } else if child.is("subscription", ns::PUBSUB_EVENT) {
                for _ in child.children() {
                    return Err(Error::ParseError("Unknown child in purge element."));
                }
                payload = Some(PubSubEvent::Subscription {
                    node: node,
                    expiry: get_attr!(child, "expiry", optional),
                    jid: get_attr!(child, "jid", optional),
                    subid: get_attr!(child, "subid", optional),
                    subscription: get_attr!(child, "subscription", optional),
                });
            } else {
                return Err(Error::ParseError("Unknown child in event element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in event element."))?)
    }
}

impl From<PubSubEvent> for Element {
    fn from(event: PubSubEvent) -> Element {
        let payload = match event {
            PubSubEvent::Configuration { node, form } => {
                Element::builder("configuration")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .append(form)
                        .build()
            },
            PubSubEvent::Delete { node, redirect } => {
                Element::builder("purge")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .append(redirect.map(|redirect| {
                             Element::builder("redirect")
                                     .ns(ns::PUBSUB_EVENT)
                                     .attr("uri", redirect)
                                     .build()
                         }))
                        .build()
            },
            PubSubEvent::EmptyItems { node } => {
                Element::builder("items")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .build()
            },
            PubSubEvent::PublishedItems { node, items } => {
                Element::builder("items")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .append(items)
                        .build()
            },
            PubSubEvent::RetractedItems { node, items } => {
                Element::builder("items")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .append(items)
                        .build()
            },
            PubSubEvent::Purge { node } => {
                Element::builder("purge")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .build()
            },
            PubSubEvent::Subscription { node, expiry, jid, subid, subscription } => {
                Element::builder("subscription")
                        .ns(ns::PUBSUB_EVENT)
                        .attr("node", node)
                        .attr("expiry", expiry.map(|expiry| expiry.to_rfc3339()))
                        .attr("jid", jid.map(String::from))
                        .attr("subid", subid)
                        .attr("subscription", subscription)
                        .build()
            },
        };
        Element::builder("event")
                .ns(ns::PUBSUB_EVENT)
                .append(payload)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='coucou'/></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::EmptyItems { node } => assert_eq!(node, String::from("coucou")),
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_items() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='coucou'><item id='test' node='huh?' publisher='test@coucou'/></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::PublishedItems { node, items } => {
                assert_eq!(node, String::from("coucou"));
                assert_eq!(items[0].id, Some(String::from("test")));
                assert_eq!(items[0].node, Some(String::from("huh?")));
                assert_eq!(items[0].publisher, Some(Jid::from_str("test@coucou").unwrap()));
                assert_eq!(items[0].payload, None);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_pep() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='something'><item><foreign xmlns='example:namespace'/></item></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::PublishedItems { node, items } => {
                assert_eq!(node, String::from("something"));
                assert_eq!(items[0].id, None);
                assert_eq!(items[0].node, None);
                assert_eq!(items[0].publisher, None);
                match items[0].payload {
                    Some(ref elem) => assert!(elem.is("foreign", "example:namespace")),
                    _ => panic!(),
                }
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_retract() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='something'><retract id='coucou'/><retract id='test'/></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::RetractedItems { node, items } => {
                assert_eq!(node, String::from("something"));
                assert_eq!(items[0], String::from("coucou"));
                assert_eq!(items[1], String::from("test"));
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_delete() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><delete node='coucou'><redirect uri='hello'/></delete></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Delete { node, redirect } => {
                assert_eq!(node, String::from("coucou"));
                assert_eq!(redirect, Some(String::from("hello")));
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_purge() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><purge node='coucou'/></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Purge { node } => {
                assert_eq!(node, String::from("coucou"));
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_configure() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><configuration node='coucou'><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field></x></configuration></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Configuration { node, form: _ } => {
                assert_eq!(node, String::from("coucou"));
                //assert_eq!(form.type_, Result_);
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><coucou node='test'/></event>".parse().unwrap();
        let error = PubSubEvent::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in event element.");
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event' coucou=''/>".parse().unwrap();
        let error = PubSubEvent::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in event element.");
    }

    #[test]
    fn test_ex221_subscription() {
        let elem: Element = r#"
<event xmlns='http://jabber.org/protocol/pubsub#event'>
  <subscription
      expiry='2006-02-28T23:59:59+00:00'
      jid='francisco@denmark.lit'
      node='princely_musings'
      subid='ba49252aaa4f5d320c24d3766f0bdcade78c78d3'
      subscription='subscribed'/>
</event>
"#.parse().unwrap();
        let event = PubSubEvent::try_from(elem.clone()).unwrap();
        match event.clone() {
            PubSubEvent::Subscription { node, expiry, jid, subid, subscription } => {
                assert_eq!(node, String::from("princely_musings"));
                assert_eq!(subid, Some(String::from("ba49252aaa4f5d320c24d3766f0bdcade78c78d3")));
                assert_eq!(subscription, Some(Subscription::Subscribed));
                assert_eq!(jid, Some(Jid::from_str("francisco@denmark.lit").unwrap()));
                assert_eq!(expiry, Some("2006-02-28T23:59:59Z".parse().unwrap()));
            },
            _ => panic!(),
        }

        let elem2: Element = event.into();
        assert_eq!(elem, elem2);
    }
}
