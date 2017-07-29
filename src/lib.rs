#![deny(missing_docs)]

//! Provides a type for Jabber IDs.
//!
//! For usage, check the documentation on the `Jid` struct.

use std::fmt;

use std::convert::Into;

use std::str::FromStr;

/// An error that signifies that a `Jid` cannot be parsed from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JidParseError {
    /// Happens when there is no domain. (really, only happens when the string is empty)
    NoDomain,
}

/// A struct representing a Jabber ID.
///
/// A Jabber ID is composed of 3 components, of which 2 are optional:
///
///  - A node/name, `node`, which is the optional part before the @.
///  - A domain, `domain`, which is the mandatory part after the @ but before the /.
///  - A resource, `resource`, which is the optional part after the /.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Jid {
    /// The node part of the Jabber ID, if it exists, else None.
    pub node: Option<String>,
    /// The domain of the Jabber ID.
    pub domain: String,
    /// The resource of the Jabber ID, if it exists, else None.
    pub resource: Option<String>,
}

impl From<Jid> for String {
    fn from(jid: Jid) -> String {
        let mut string = String::new();
        if let Some(ref node) = jid.node {
            string.push_str(node);
            string.push('@');
        }
        string.push_str(&jid.domain);
        if let Some(ref resource) = jid.resource {
            string.push('/');
            string.push_str(resource);
        }
        string
    }
}

impl fmt::Debug for Jid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "JID({})", self)?;
        Ok(())
    }
}

impl fmt::Display for Jid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(String::from(self.clone()).as_ref())?;
        Ok(())
    }
}

enum ParserState {
    Node,
    Domain,
    Resource
}

impl FromStr for Jid {
    type Err = JidParseError;

    fn from_str(s: &str) -> Result<Jid, JidParseError> {
        // TODO: very naive, may need to do it differently
        let iter = s.chars();
        let mut buf = String::with_capacity(s.len());
        let mut state = ParserState::Node;
        let mut node = None;
        let mut domain = None;
        let mut resource = None;
        for c in iter {
            match state {
                ParserState::Node => {
                    match c {
                        '@' => {
                            state = ParserState::Domain;
                            node = Some(buf.clone()); // TODO: performance tweaks, do not need to copy it
                            buf.clear();
                        },
                        '/' => {
                            state = ParserState::Resource;
                            domain = Some(buf.clone()); // TODO: performance tweaks
                            buf.clear();
                        },
                        c => {
                            buf.push(c);
                        },
                    }
                },
                ParserState::Domain => {
                    match c {
                        '/' => {
                            state = ParserState::Resource;
                            domain = Some(buf.clone()); // TODO: performance tweaks
                            buf.clear();
                        },
                        c => {
                            buf.push(c);
                        },
                    }
                },
                ParserState::Resource => {
                    buf.push(c);
                },
            }
        }
        if !buf.is_empty() {
            match state {
                ParserState::Node => {
                    domain = Some(buf);
                },
                ParserState::Domain => {
                    domain = Some(buf);
                },
                ParserState::Resource => {
                    resource = Some(buf);
                },
            }
        }
        Ok(Jid {
            node: node,
            domain: domain.ok_or(JidParseError::NoDomain)?,
            resource: resource,
        })
    }
}

impl Jid {
    /// Constructs a Jabber ID containing all three components.
    ///
    /// This is of the form `node`@`domain`/`resource`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::full("node", "domain", "resource");
    ///
    /// assert_eq!(jid.node, Some("node".to_owned()));
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, Some("resource".to_owned()));
    /// ```
    pub fn full<NS, DS, RS>(node: NS, domain: DS, resource: RS) -> Jid
        where NS: Into<String>
            , DS: Into<String>
            , RS: Into<String> {
        Jid {
            node: Some(node.into()),
            domain: domain.into(),
            resource: Some(resource.into()),
        }
    }

    /// Constructs a Jabber ID containing only the `node` and `domain` components.
    ///
    /// This is of the form `node`@`domain`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::bare("node", "domain");
    ///
    /// assert_eq!(jid.node, Some("node".to_owned()));
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, None);
    /// ```
    pub fn bare<NS, DS>(node: NS, domain: DS) -> Jid
        where NS: Into<String>
            , DS: Into<String> {
        Jid {
            node: Some(node.into()),
            domain: domain.into(),
            resource: None,
        }
    }

    /// Returns a new Jabber ID from the current one with only node and domain.
    ///
    /// This is of the form `node`@`domain`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::full("node", "domain", "resource").into_bare_jid();
    ///
    /// assert_eq!(jid.node, Some("node".to_owned()));
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, None);
    /// ```
    pub fn into_bare_jid(self) -> Jid {
        Jid {
            node: self.node,
            domain: self.domain,
            resource: None,
        }
    }

    /// Constructs a Jabber ID containing only a `domain`.
    ///
    /// This is of the form `domain`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::domain("domain");
    ///
    /// assert_eq!(jid.node, None);
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, None);
    /// ```
    pub fn domain<DS>(domain: DS) -> Jid
        where DS: Into<String> {
        Jid {
            node: None,
            domain: domain.into(),
            resource: None,
        }
    }

    /// Returns a new Jabber ID from the current one with only domain.
    ///
    /// This is of the form `domain`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::full("node", "domain", "resource").into_domain_jid();
    ///
    /// assert_eq!(jid.node, None);
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, None);
    /// ```
    pub fn into_domain_jid(self) -> Jid {
        Jid {
            node: None,
            domain: self.domain,
            resource: None,
        }
    }

    /// Constructs a Jabber ID containing the `domain` and `resource` components.
    ///
    /// This is of the form `domain`/`resource`.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::domain_with_resource("domain", "resource");
    ///
    /// assert_eq!(jid.node, None);
    /// assert_eq!(jid.domain, "domain".to_owned());
    /// assert_eq!(jid.resource, Some("resource".to_owned()));
    /// ```
    pub fn domain_with_resource<DS, RS>(domain: DS, resource: RS) -> Jid
        where DS: Into<String>
            , RS: Into<String> {
        Jid {
            node: None,
            domain: domain.into(),
            resource: Some(resource.into()),
        }
    }

    /// Constructs a new Jabber ID from an existing one, with the node swapped out with a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::domain("domain");
    ///
    /// assert_eq!(jid.node, None);
    ///
    /// let new_jid = jid.with_node("node");
    ///
    /// assert_eq!(new_jid.node, Some("node".to_owned()));
    /// ```
    pub fn with_node<S>(&self, node: S) -> Jid
        where S: Into<String> {
        Jid {
            node: Some(node.into()),
            domain: self.domain.clone(),
            resource: self.resource.clone(),
        }
    }

    /// Constructs a new Jabber ID from an existing one, with the domain swapped out with a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::domain("domain");
    ///
    /// assert_eq!(jid.domain, "domain");
    ///
    /// let new_jid = jid.with_domain("new_domain");
    ///
    /// assert_eq!(new_jid.domain, "new_domain");
    /// ```
    pub fn with_domain<S>(&self, domain: S) -> Jid
        where S: Into<String> {
        Jid {
            node: self.node.clone(),
            domain: domain.into(),
            resource: self.resource.clone(),
        }
    }

    /// Constructs a new Jabber ID from an existing one, with the resource swapped out with a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// use jid::Jid;
    ///
    /// let jid = Jid::domain("domain");
    ///
    /// assert_eq!(jid.resource, None);
    ///
    /// let new_jid = jid.with_resource("resource");
    ///
    /// assert_eq!(new_jid.resource, Some("resource".to_owned()));
    /// ```
    pub fn with_resource<S>(&self, resource: S) -> Jid
        where S: Into<String> {
        Jid {
            node: self.node.clone(),
            domain: self.domain.clone(),
            resource: Some(resource.into()),
        }
    }

}

#[cfg(feature = "minidom")]
extern crate minidom;

#[cfg(feature = "minidom")]
use minidom::IntoAttributeValue;

#[cfg(feature = "minidom")]
impl IntoAttributeValue for Jid {
    fn into_attribute_value(self) -> Option<String> {
        Some(String::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn can_parse_jids() {
        assert_eq!(Jid::from_str("a@b.c/d"), Ok(Jid::full("a", "b.c", "d")));
        assert_eq!(Jid::from_str("a@b.c"), Ok(Jid::bare("a", "b.c")));
        assert_eq!(Jid::from_str("b.c"), Ok(Jid::domain("b.c")));

        assert_eq!(Jid::from_str(""), Err(JidParseError::NoDomain));

        assert_eq!(Jid::from_str("a/b@c"), Ok(Jid::domain_with_resource("a", "b@c")));
    }

    #[test]
    fn serialise() {
        assert_eq!(String::from(Jid::full("a", "b", "c")), String::from("a@b/c"));
    }

    #[cfg(feature = "minidom")]
    #[test]
    fn minidom() {
        let elem: minidom::Element = "<message from='a@b/c'/>".parse().unwrap();
        let to: Jid = elem.attr("from").unwrap().parse().unwrap();
        assert_eq!(to, Jid::full("a", "b", "c"));
    }
}
