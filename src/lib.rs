//! A crate parsing common XMPP elements into Rust structures.
//!
//! Each module implements the [`TryFrom<Element>`] trait, which takes a
//! minidom [`Element`] and returns a `Result` whose value is `Ok` if the
//! element parsed correctly, `Err(error::Error)` otherwise.
//!
//! The returned structure can be manipuled as any Rust structure, with each
//! field being public.  You can also create the same structure manually, with
//! some having `new()` and `with_*()` helper methods to create them.
//!
//! Once you are happy with your structure, you can serialise it back to an
//! [`Element`], using either `From` or `Into<Element>`, which give you what
//! you want to be sending on the wire.
//!
//! [`TryFrom<Element>`]: ../try_from/trait.TryFrom.html
//! [`Element`]: ../minidom/element/struct.Element.html

// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate minidom;
extern crate jid;
extern crate base64;
extern crate digest;
extern crate sha_1;
extern crate sha2;
extern crate sha3;
extern crate blake2;
extern crate chrono;
extern crate try_from;

macro_rules! get_attr {
    ($elem:ident, $attr:tt, $type:tt) => (
        get_attr!($elem, $attr, $type, value, value.parse()?)
    );
    ($elem:ident, $attr:tt, optional, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => Some($func),
            None => None,
        }
    );
    ($elem:ident, $attr:tt, required, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => $func,
            None => return Err(Error::ParseError(concat!("Required attribute '", $attr, "' missing."))),
        }
    );
    ($elem:ident, $attr:tt, default, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => $func,
            None => Default::default(),
        }
    );
}

macro_rules! generate_attribute {
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+,}) => (
        generate_attribute!($elem, $name, {$($a => $b),+});
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+,}, Default = $default:ident) => (
        generate_attribute!($elem, $name, {$($a => $b),+}, Default = $default);
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+}) => (
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
                #[doc=$b]
                #[doc="value for this attribute."]
                $a
            ),+
        }
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                Ok(match s {
                    $($b => $elem::$a),+,
                    _ => return Err(Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
                })
            }
        }
        impl IntoAttributeValue for $elem {
            fn into_attribute_value(self) -> Option<String> {
                Some(String::from(match self {
                    $($elem::$a => $b),+
                }))
            }
        }
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+}, Default = $default:ident) => (
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
                #[doc=$b]
                #[doc="value for this attribute."]
                $a
            ),+
        }
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                Ok(match s {
                    $($b => $elem::$a),+,
                    _ => return Err(Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
                })
            }
        }
        impl IntoAttributeValue for $elem {
            #[allow(unreachable_patterns)]
            fn into_attribute_value(self) -> Option<String> {
                Some(String::from(match self {
                    $elem::$default => return None,
                    $($elem::$a => $b),+
                }))
            }
        }
        impl Default for $elem {
            fn default() -> $elem {
                $elem::$default
            }
        }
    );
}

macro_rules! check_self {
    ($elem:ident, $name:tt, $ns:expr) => (
        if !$elem.is($name, $ns) {
            return Err(Error::ParseError(concat!("This is not a ", $name, " element.")));
        }
    );
    ($elem:ident, $name:tt, $ns:expr, $pretty_name:tt) => (
        if !$elem.is($name, $ns) {
            return Err(Error::ParseError(concat!("This is not a ", $pretty_name, " element.")));
        }
    );
}

macro_rules! check_ns_only {
    ($elem:ident, $name:tt, $ns:expr) => (
        if !$elem.has_ns($ns) {
            return Err(Error::ParseError(concat!("This is not a ", $name, " element.")));
        }
    );
}

macro_rules! check_no_children {
    ($elem:ident, $name:tt) => (
        for _ in $elem.children() {
            return Err(Error::ParseError(concat!("Unknown child in ", $name, " element.")));
        }
    );
}

macro_rules! check_no_attributes {
    ($elem:ident, $name:tt) => (
        check_no_unknown_attributes!($elem, $name, []);
    );
}

macro_rules! check_no_unknown_attributes {
    ($elem:ident, $name:tt, [$($attr:tt),*]) => (
        for (_attr, _) in $elem.attrs() {
            $(
            if _attr == $attr {
                continue;
            }
            )*
            return Err(Error::ParseError(concat!("Unknown attribute in ", $name, " element.")));
        }
    );
}

macro_rules! generate_empty_element {
    ($elem:ident, $name:tt, $ns:expr) => (
        // TODO: Find a better way to concatenate doc.
        #[doc="Structure representing a "]
        #[doc=$name]
        #[doc=" element."]
        #[derive(Debug, Clone)]
        pub struct $elem;

        impl TryFrom<Element> for $elem {
            type Err = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_attributes!(elem, $name);
                Ok($elem)
            }
        }

        impl From<$elem> for Element {
            fn from(_: $elem) -> Element {
                Element::builder("attention")
                        .ns($ns)
                        .build()
            }
        }
    );
}

macro_rules! generate_id {
    ($elem:ident) => (
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $elem(pub String);
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                // TODO: add a way to parse that differently when needed.
                Ok($elem(String::from(s)))
            }
        }
        impl IntoAttributeValue for $elem {
            fn into_attribute_value(self) -> Option<String> {
                Some(self.0)
            }
        }
    );
}

macro_rules! generate_elem_id {
    ($elem:ident, $name:tt, $ns:expr) => (
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $elem(pub String);
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                // TODO: add a way to parse that differently when needed.
                Ok($elem(String::from(s)))
            }
        }
        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        .append(elem.0)
                        .build()
            }
        }
    );
}

/// Error type returned by every parser on failure.
pub mod error;
/// XML namespace definitions used through XMPP.
pub mod ns;

#[cfg(test)]
/// Namespace-aware comparison for tests
mod compare_elements;

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod message;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod presence;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod iq;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod stanza_error;

/// RFC 6121: Extensible Messaging and Presence Protocol (XMPP): Instant Messaging and Presence
pub mod roster;

/// XEP-0004: Data Forms
pub mod data_forms;

/// XEP-0030: Service Discovery
pub mod disco;

/// XEP-0045: Multi-User Chat
pub mod muc;

/// XEP-0047: In-Band Bytestreams
pub mod ibb;

/// XEP-0059: Result Set Management
pub mod rsm;

/// XEP-0060: Publish-Subscribe
pub mod pubsub;

/// XEP-0077: In-Band Registration
pub mod ibr;

/// XEP-0085: Chat State Notifications
pub mod chatstates;

/// XEP-0092: Software Version
pub mod version;

/// XEP-0115: Entity Capabilities
pub mod caps;

/// XEP-0166: Jingle
pub mod jingle;

/// XEP-0184: Message Delivery Receipts
pub mod receipts;

/// XEP-0191: Blocking Command
pub mod blocking;

/// XEP-0199: XMPP Ping
pub mod ping;

/// XEP-0203: Delayed Delivery
pub mod delay;

/// XEP-0221: Data Forms Media Element
pub mod media_element;

/// XEP-0224: Attention
pub mod attention;

/// XEP-0234: Jingle File Transfer
pub mod jingle_ft;

/// XEP-0260: Jingle SOCKS5 Bytestreams Transport Method
pub mod jingle_s5b;

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub mod jingle_ibb;

/// XEP-0297: Stanza Forwarding
pub mod forwarding;

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub mod hashes;

/// XEP-0308: Last Message Correction
pub mod message_correct;

/// XEP-0313: Message Archive Management
pub mod mam;

/// XEP-0319: Last User Interaction in Presence
pub mod idle;

/// XEP-0353: Jingle Message Initiation
pub mod jingle_message;

/// XEP-0359: Unique and Stable Stanza IDs
pub mod stanza_id;

/// XEP-0380: Explicit Message Encryption
pub mod eme;

/// XEP-0390: Entity Capabilities 2.0
pub mod ecaps2;
