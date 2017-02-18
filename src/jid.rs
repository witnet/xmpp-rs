use std::fmt;

use std::convert::Into;

use std::str::FromStr;

use std::string::ToString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JidParseError {
    NoDomain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jid {
    pub node: Option<String>,
    pub domain: String,
    pub resource: Option<String>,
}

impl fmt::Display for Jid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: may need escaping
        if let Some(ref node) = self.node {
            write!(fmt, "{}@", node)?;
        }
        write!(fmt, "{}", self.domain)?;
        if let Some(ref resource) = self.resource {
            write!(fmt, "/{}", resource)?;
        }
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
        let mut iter = s.chars();
        let mut buf = String::new();
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

    pub fn bare<NS, DS>(node: NS, domain: DS) -> Jid
        where NS: Into<String>
            , DS: Into<String> {
        Jid {
            node: Some(node.into()),
            domain: domain.into(),
            resource: None,
        }
    }

    pub fn domain<DS>(domain: DS) -> Jid
        where DS: Into<String> {
        Jid {
            node: None,
            domain: domain.into(),
            resource: None,
        }
    }

    pub fn domain_with_resource<DS, RS>(domain: DS, resource: RS) -> Jid
        where DS: Into<String>
            , RS: Into<String> {
        Jid {
            node: None,
            domain: domain.into(),
            resource: Some(resource.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_jids() {
        assert_eq!(Jid::from_str("a@b.c/d"), Ok(Jid::full("a", "b.c", "d")));
        assert_eq!(Jid::from_str("a@b.c"), Ok(Jid::bare("a", "b.c")));
        assert_eq!(Jid::from_str("b.c"), Ok(Jid::domain("b.c")));

        assert_eq!(Jid::from_str(""), Err(JidParseError::NoDomain));

        assert_eq!(Jid::from_str("a/b@c"), Ok(Jid::domain_with_resource("a", "b@c")));
    }
}
