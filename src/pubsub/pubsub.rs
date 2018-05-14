// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use ns;

use data_forms::DataForm;

use pubsub::{NodeName, ItemId, Subscription, SubscriptionId};

// TODO: a better solution would be to split this into a query and a result elements, like for
// XEP-0030.
generate_element_with_children!(
    Affiliations, "affiliations", PUBSUB,
    attributes: [
        node: Option<NodeName> = "node" => optional,
    ],
    children: [
        affiliations: Vec<Affiliation> = ("affiliation", PUBSUB) => Affiliation
    ]
);

generate_attribute!(
    AffiliationAttribute, "affiliation", {
        Member => "member",
        None => "none",
        Outcast => "outcast",
        Owner => "owner",
        Publisher => "publisher",
        PublishOnly => "publish-only",
    }
);

generate_element_with_only_attributes!(
    Affiliation, "affiliation", PUBSUB, [
        node: NodeName = "node" => required,
        affiliation: AffiliationAttribute = "affiliation" => required,
    ]
);

generate_element_with_children!(
    Configure, "configure", PUBSUB,
    child: (
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    )
);

generate_element_with_only_attributes!(
    Create, "create", PUBSUB, [
        node: Option<NodeName> = "node" => optional,
    ]
);

generate_element_with_only_attributes!(
    Default, "default", PUBSUB, [
        node: Option<NodeName> = "node" => optional,
        // TODO: do we really want to support collection nodes?
        // type: String = "type" => optional,
    ]
);

generate_element_with_children!(
    Items, "items", PUBSUB,
    attributes: [
        // TODO: should be an xs:positiveInteger, that is, an unbounded int ≥ 1.
        max_items: Option<u32> = "max_items" => optional,
        node: NodeName = "node" => required,
        subid: Option<SubscriptionId> = "subid" => optional,
    ],
    children: [
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

#[derive(Debug, Clone)]
pub struct Item {
    payload: Option<Element>,
    id: Option<ItemId>,
}

impl TryFrom<Element> for Item {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Item, Error> {
        check_self!(elem, "item", PUBSUB);
        check_no_unknown_attributes!(elem, "item", ["id"]);
        let mut payloads = elem.children().cloned().collect::<Vec<_>>();
        let payload = payloads.pop();
        if !payloads.is_empty() {
            return Err(Error::ParseError("More than a single payload in item element."));
        }
        Ok(Item {
            payload,
            id: get_attr!(elem, "id", optional),
        })
    }
}

impl From<Item> for Element {
    fn from(item: Item) -> Element {
        Element::builder("item")
                .ns(ns::PUBSUB)
                .attr("id", item.id)
                .append(item.payload)
                .build()
    }
}

generate_element_with_children!(
    Options, "options", PUBSUB,
    attributes: [
        jid: Jid = "jid" => required,
        node: Option<NodeName> = "node" => optional,
        subid: Option<SubscriptionId> = "subid" => optional,
    ],
    child: (
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    )
);

generate_element_with_children!(
    Publish, "publish", PUBSUB,
    attributes: [
        node: NodeName = "node" => required,
    ],
    children: [
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

generate_element_with_children!(
    PublishOptions, "publish-options", PUBSUB,
    child: (
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    )
);

generate_attribute!(Notify, "notify", bool);

generate_element_with_children!(
    Retract, "retract", PUBSUB,
    attributes: [
        node: NodeName = "node" => required,
        notify: Notify = "notify" => default,
    ],
    children: [
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    required: bool,
}

impl TryFrom<Element> for SubscribeOptions {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Self, Error> {
        check_self!(elem, "subscribe-options", PUBSUB);
        check_no_attributes!(elem, "subscribe-options");
        let mut required = false;
        for child in elem.children() {
            if child.is("required", ns::PUBSUB) {
                if required {
                    return Err(Error::ParseError("More than one required element in subscribe-options."));
                }
                required = true;
            } else {
                return Err(Error::ParseError("Unknown child in subscribe-options element."));
            }
        }
        Ok(SubscribeOptions { required })
    }
}

impl From<SubscribeOptions> for Element {
    fn from(subscribe_options: SubscribeOptions) -> Element {
        Element::builder("subscribe-options")
            .ns(ns::PUBSUB)
            .append(if subscribe_options.required {
                 vec!(Element::builder("required")
                     .ns(ns::PUBSUB)
                     .build())
             } else {
                 vec!()
             })
            .build()
    }
}

generate_element_with_only_attributes!(
    Subscribe, "subscribe", PUBSUB, [
        jid: Jid = "jid" => required,
        node: Option<NodeName> = "node" => optional,
    ]
);

generate_element_with_children!(
    Subscriptions, "subscriptions", PUBSUB,
    attributes: [
        node: Option<NodeName> = "node" => optional,
    ],
    children: [
        subscription: Vec<SubscriptionElem> = ("subscription", PUBSUB) => SubscriptionElem
    ]
);

generate_element_with_children!(
    SubscriptionElem, "subscription", PUBSUB,
    attributes: [
        jid: Jid = "jid" => required,
        node: Option<NodeName> = "node" => optional,
        subid: Option<SubscriptionId> = "subid" => optional,
        subscription: Option<Subscription> = "subscription" => optional,
    ],
    child: (
        subscribe_options: Option<SubscribeOptions> = ("subscribe-options", PUBSUB) => SubscribeOptions
    )
);

generate_element_with_only_attributes!(
    Unsubscribe, "unsubscribe", PUBSUB, [
        jid: Jid = "jid" => required,
        node: Option<NodeName> = "node" => optional,
        subid: Option<SubscriptionId> = "subid" => optional,
    ]
);

#[derive(Debug, Clone)]
pub enum PubSub {
    Create {
        create: Create,
        configure: Option<Configure>
    },
    Publish {
        publish: Publish,
        publish_options: Option<PublishOptions>
    },
    Affiliations(Affiliations),
    Default(Default),
    Items(Items),
    Retract(Retract),
    Subscription(SubscriptionElem),
    Subscriptions(Subscriptions),
    Unsubscribe(Unsubscribe),
}

impl TryFrom<Element> for PubSub {
    type Err = Error;

    fn try_from(elem: Element) -> Result<PubSub, Error> {
        check_self!(elem, "pubsub", PUBSUB);
        check_no_attributes!(elem, "pubsub");

        let mut payload = None;
        for child in elem.children() {
            if child.is("create", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let create = Create::try_from(child.clone())?;
                payload = Some(PubSub::Create { create, configure: None });
            } else if child.is("configure", ns::PUBSUB) {
                if let Some(PubSub::Create { create, configure }) = payload {
                    if configure.is_some() {
                        return Err(Error::ParseError("Configure is already defined in pubsub element."));
                    }
                    let configure = Some(Configure::try_from(child.clone())?);
                    payload = Some(PubSub::Create { create, configure });
                } else {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
            } else if child.is("publish", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let publish = Publish::try_from(child.clone())?;
                payload = Some(PubSub::Publish { publish, publish_options: None });
            } else if child.is("publish-options", ns::PUBSUB) {
                if let Some(PubSub::Publish { publish, publish_options }) = payload {
                    if publish_options.is_some() {
                        return Err(Error::ParseError("Publish-options are already defined in pubsub element."));
                    }
                    let publish_options = Some(PublishOptions::try_from(child.clone())?);
                    payload = Some(PubSub::Publish { publish, publish_options });
                } else {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
            } else if child.is("affiliations", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let affiliations = Affiliations::try_from(child.clone())?;
                payload = Some(PubSub::Affiliations(affiliations));
            } else if child.is("default", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let default = Default::try_from(child.clone())?;
                payload = Some(PubSub::Default(default));
            } else if child.is("items", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let items = Items::try_from(child.clone())?;
                payload = Some(PubSub::Items(items));
            } else if child.is("retract", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let retract = Retract::try_from(child.clone())?;
                payload = Some(PubSub::Retract(retract));
            } else if child.is("subscription", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let subscription = SubscriptionElem::try_from(child.clone())?;
                payload = Some(PubSub::Subscription(subscription));
            } else if child.is("subscriptions", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let subscriptions = Subscriptions::try_from(child.clone())?;
                payload = Some(PubSub::Subscriptions(subscriptions));
            } else if child.is("unsubscribe", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError("Payload is already defined in pubsub element."));
                }
                let unsubscribe = Unsubscribe::try_from(child.clone())?;
                payload = Some(PubSub::Unsubscribe(unsubscribe));
            } else {
                return Err(Error::ParseError("Unknown child in pubsub element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in pubsub element."))?)
    }
}

impl From<PubSub> for Element {
    fn from(pubsub: PubSub) -> Element {
        Element::builder("pubsub")
            .ns(ns::PUBSUB)
            .append(match pubsub {
                 PubSub::Create { create, configure } => {
                     let mut elems = vec!(Element::from(create));
                     if let Some(configure) = configure {
                         elems.push(Element::from(configure));
                     }
                     elems
                 },
                 PubSub::Publish { publish, publish_options } => {
                     let mut elems = vec!(Element::from(publish));
                     if let Some(publish_options) = publish_options {
                         elems.push(Element::from(publish_options));
                     }
                     elems
                 },
                 PubSub::Affiliations(affiliations) => vec!(Element::from(affiliations)),
                 PubSub::Default(default) => vec!(Element::from(default)),
                 PubSub::Items(items) => vec!(Element::from(items)),
                 PubSub::Retract(retract) => vec!(Element::from(retract)),
                 PubSub::Subscription(subscription) => vec!(Element::from(subscription)),
                 PubSub::Subscriptions(subscriptions) => vec!(Element::from(subscriptions)),
                 PubSub::Unsubscribe(unsubscribe) => vec!(Element::from(unsubscribe)),
             })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn create() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));

        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create node='coucou'/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert_eq!(&create.node.unwrap().0, "coucou");
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn create_and_configure() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/><configure/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn publish() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish { publish, publish_options } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn publish_with_publish_options() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/><publish-options/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish { publish, publish_options } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn invalid_empty_pubsub() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'/>".parse().unwrap();
        let error = PubSub::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No payload in pubsub element.");
    }

    #[test]
    fn publish_option() {
        let elem: Element = "<publish-options xmlns='http://jabber.org/protocol/pubsub'><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#publish-options</value></field></x></publish-options>".parse().unwrap();
        let publish_options = PublishOptions::try_from(elem).unwrap();
        assert_eq!(&publish_options.form.unwrap().form_type.unwrap(), "http://jabber.org/protocol/pubsub#publish-options");
    }

    #[test]
    fn subscribe_options() {
        let elem1: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'/>".parse().unwrap();
        let subscribe_options1 = SubscribeOptions::try_from(elem1).unwrap();
        assert_eq!(subscribe_options1.required, false);

        let elem2: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'><required/></subscribe-options>".parse().unwrap();
        let subscribe_options2 = SubscribeOptions::try_from(elem2).unwrap();
        assert_eq!(subscribe_options2.required, true);
    }
}
