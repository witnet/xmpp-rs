// Copyright (c) ???
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::DataForm;
use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::pubsub::NodeName;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

generate_element!(
    /// Request to configure a node.
    Configure, "configure", PUBSUB_OWNER,
    attributes: [
        /// The node to be configured.
        node: Option<NodeName> = "node",
    ],
    children: [
        /// The form to configure it.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

/// Main payload used to communicate with a PubSubOwner service.
///
/// `<pubsub xmlns="http://jabber.org/protocol/pubsub#owner"/>`
#[derive(Debug, Clone)]
pub enum PubSubOwner {
    /// Request to configure a node, with optional suggested name and suggested configuration.
    Configure {
        /// The configure request for the new node.
        configure: Configure,
    },
}

impl IqGetPayload for PubSubOwner {}
impl IqSetPayload for PubSubOwner {}
impl IqResultPayload for PubSubOwner {}

impl TryFrom<Element> for PubSubOwner {
    type Error = Error;

    fn try_from(elem: Element) -> Result<PubSubOwner, Error> {
        check_self!(elem, "pubsub", PUBSUB_OWNER);
        check_no_attributes!(elem, "pubsub");

        let mut payload = None;
        for child in elem.children() {
            if child.is("configure", ns::PUBSUB_OWNER) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub owner element.",
                    ));
                }
                let configure = Configure::try_from(child.clone())?;
                payload = Some(PubSubOwner::Configure {
                    configure: configure,
                });
            } else {
                return Err(Error::ParseError("Unknown child in pubsub element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in pubsub element."))?)
    }
}

impl From<PubSubOwner> for Element {
    fn from(pubsub: PubSubOwner) -> Element {
        Element::builder("pubsub", ns::PUBSUB_OWNER)
            .append_all(match pubsub {
                PubSubOwner::Configure { configure } => vec![Element::from(configure)],
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_forms::{DataForm, DataFormType, Field, FieldType};
    use std::str::FromStr;

    #[test]
    fn configure() {
        // XXX: Do we want xmpp-parsers to always specify the field type in the output Element?
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><configure node='foo'><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field><field var='pubsub#access_model' type='list-single'><value>whitelist</value></field></x></configure></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Configure {
            configure: Configure {
                node: Some(NodeName(String::from("foo"))),
                form: Some(DataForm {
                    type_: DataFormType::Submit,
                    form_type: Some(String::from(ns::PUBSUB_CONFIGURE)),
                    title: None,
                    instructions: None,
                    fields: vec![Field {
                        var: String::from("pubsub#access_model"),
                        type_: FieldType::ListSingle,
                        label: None,
                        required: false,
                        options: vec![],
                        values: vec![String::from("whitelist")],
                        media: vec![],
                    }],
                }),
            },
        };

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_serialize_configure() {
        let reference: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><configure node='foo'><x xmlns='jabber:x:data' type='submit'/></configure></pubsub>"
        .parse()
        .unwrap();

        let elem: Element = "<x xmlns='jabber:x:data' type='submit'/>".parse().unwrap();

        let form = DataForm::try_from(elem).unwrap();

        let configure = PubSubOwner::Configure {
            configure: Configure {
                node: Some(NodeName(String::from("foo"))),
                form: Some(form),
            },
        };
        let serialized: Element = configure.into();
        assert_eq!(serialized, reference);
    }
}
