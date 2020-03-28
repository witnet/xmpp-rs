use lazy_static::lazy_static;
use string_interner::{StringInterner, Sym};
use std::collections::BTreeMap;
use std::sync::{Mutex, Weak};

lazy_static! {
    static ref NAME_INTERNER: Mutex<StringInterner<Sym>> = Mutex::new(StringInterner::default());
    static ref NS_INTERNER: Mutex<StringInterner<Sym>> = Mutex::new(StringInterner::default());
    static ref PREFIX_INTERNER: Mutex<StringInterner<Sym>> = Mutex::new(StringInterner::default());
    static ref ATTR_INTERNER: Mutex<StringInterner<Sym>> = Mutex::new(StringInterner::default());

    static ref NO_NS: Sym = NS_INTERNER.lock().unwrap().get_or_intern("");
}

type LocalName = Sym;
type Namespace = Sym;
type Prefix = Option<Sym>;
type Attribute = (Prefix, Sym);

#[derive(Debug, Clone)]
struct Element {
    local_name: LocalName,
    namespace: Namespace,
    attributes: BTreeMap<Attribute, String>,
    defined_prefixes: BTreeMap<Prefix, Namespace>,
    parent: Weak<Element>,
}

impl Element {
    pub fn new(local_name: &str, namespace: &str) -> Element {
        let local_name = NAME_INTERNER.lock().unwrap().get_or_intern(local_name);
        let namespace = NS_INTERNER.lock().unwrap().get_or_intern(namespace);
        Element {
            local_name,
            namespace,
            attributes: BTreeMap::new(),
            defined_prefixes: BTreeMap::new(),
            parent: Weak::new(),
        }
    }

    pub fn prefix(mut self, prefix: Option<&str>, namespace: &str) -> Element {
        let prefix = prefix.map(|prefix| {
            PREFIX_INTERNER.lock().unwrap().get_or_intern(prefix)
        });
        let namespace = NS_INTERNER.lock().unwrap().get_or_intern(namespace);
        self.defined_prefixes.insert(prefix, namespace);
        self
    }

    pub fn attr<V: Into<String>>(mut self, attr: &str, value: V) -> Element {
        match attr.splitn(2, ':').collect::<Vec<_>>()[..] {
            [prefix, attr] => {
                let prefix = PREFIX_INTERNER.lock().unwrap().get_or_intern(prefix);
                let attr = ATTR_INTERNER.lock().unwrap().get_or_intern(attr);
                self.attributes.insert((Some(prefix), attr), value.into());
            }
            [attr] => {
                let attr = ATTR_INTERNER.lock().unwrap().get_or_intern(attr);
                self.attributes.insert((None, attr), value.into());
            }
            _ => unreachable!(),
        }
        self
    }
}

use core::fmt;
impl fmt::Display for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "<")?;
        let prefix_interner = PREFIX_INTERNER.lock().unwrap();
        match self.parent.upgrade() {
            None => {
                for (prefix, ns) in &self.defined_prefixes {
                    if self.namespace == *ns {
                        match prefix {
                            None => (),
                            Some(prefix) => write!(fmt, "{}:", prefix_interner.resolve(*prefix).unwrap())?,
                        }
                        break;
                    }
                }
            }
            Some(parent) => {
                todo!("Don’t duplicate stuff from parent: {}", parent)
            }
        }
        write!(fmt, "{}", NAME_INTERNER.lock().unwrap().resolve(self.local_name).unwrap())?;
        match self.parent.upgrade() {
            None => {
                let ns_interner = NS_INTERNER.lock().unwrap();
                for (prefix, ns) in &self.defined_prefixes {
                    let prefix = match prefix {
                        None => String::new(),
                        Some(prefix) => format!(":{}", prefix_interner.resolve(*prefix).unwrap()),
                    };
                    let ns = ns_interner.resolve(*ns).unwrap();
                    write!(fmt, " xmlns{}='{}'", prefix, ns)?;
                }
            }
            Some(parent) => {
                todo!("Don’t duplicate stuff from parent: {}", parent)
            }
        }
        {
            let attr_interner = ATTR_INTERNER.lock().unwrap();
            for ((prefix, attr), value) in &self.attributes {
                let prefix = match prefix {
                    None => String::new(),
                    Some(prefix) => format!("{}:", prefix_interner.resolve(*prefix).unwrap()),
                };
                let attr = attr_interner.resolve(*attr).unwrap();
                write!(fmt, " {}{}='{}'", prefix, attr, value)?;
            }
        }
        write!(fmt, ">")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        assert_eq!(std::mem::size_of::<Element>(), 64);
    }

    #[test]
    fn stream_stream() {
        let elem = Element::new("stream", "http://etherx.jabber.org/streams")
            .prefix(Some("stream"), "http://etherx.jabber.org/streams")
            .prefix(None, "jabber:client")
            .attr("xml:lang", "en")
            .attr("version", "1.0")
            .attr("id", "628859e4-caeb-4c4e-9590-752b4f2938c9")
            .attr("from", "linkmauve.fr");
        let data = format!("{}", elem);
        assert_eq!(data, "<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' version='1.0' id='628859e4-caeb-4c4e-9590-752b4f2938c9' from='linkmauve.fr' xml:lang='en'>");
    }

    #[test]
    fn send() {
        use std::thread;
        let child = thread::spawn(|| {
            Element::new("message", "jabber:client")
                .prefix(None, "jabber:client")
        });
        let elem = child.join().unwrap();
        let data = format!("{}", elem);
        assert_eq!(data, "<message xmlns='jabber:client'>");
    }
}
