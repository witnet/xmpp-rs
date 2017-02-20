extern crate xml;

pub mod error;
pub mod attribute;

use std::io::prelude::*;

use std::convert::AsRef;

use std::iter::Iterator;

use std::slice;

use std::fmt;

use xml::reader::{XmlEvent as ReaderEvent, EventReader};
use xml::writer::{XmlEvent as WriterEvent, EventWriter};
use xml::name::Name;
use xml::namespace::NS_NO_PREFIX;

use error::Error;

use attribute::Attribute;

#[derive(Clone, PartialEq, Eq)]
pub struct Element {
    name: String,
    namespace: Option<String>,
    attributes: Vec<Attribute>,
    children: Vec<Fork>,
}

impl fmt::Debug for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref ns) = self.namespace {
            write!(fmt, "<{{{}}}{}", ns, self.name)?;
        }
        else {
            write!(fmt, "<{}", self.name)?;
        }
        for attr in &self.attributes {
            write!(fmt, " {}", attr)?;
        }
        write!(fmt, ">")?;
        for child in &self.children {
            match *child {
                Fork::Element(ref e) => {
                    write!(fmt, "{:?}", e)?;
                },
                Fork::Text(ref s) => {
                    write!(fmt, "{}", s)?;
                },
            }
        }
        write!(fmt, "</{}>", self.name)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fork {
    Element(Element),
    Text(String),
}

impl Element {
    pub fn new(name: String, namespace: Option<String>, attributes: Vec<Attribute>) -> Element {
        Element {
            name: name,
            namespace: namespace,
            attributes: attributes,
            children: Vec::new(),
        }
    }

    pub fn builder<S: Into<String>>(name: S) -> ElementBuilder {
        ElementBuilder {
            name: name.into(),
            namespace: None,
            attributes: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ns(&self) -> Option<&str> {
        self.namespace.as_ref()
                      .map(String::as_ref)
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        for attr in &self.attributes {
            if attr.name == name {
                return Some(&attr.value);
            }
        }
        None
    }

    pub fn is<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> bool {
        let ns = self.namespace.as_ref().map(String::as_ref);
        self.name == name.as_ref() && ns == Some(namespace.as_ref())
    }

    pub fn from_reader<R: Read>(reader: &mut EventReader<R>) -> Result<Element, Error> {
        loop {
            let e = reader.next()?;
            match e {
                ReaderEvent::StartElement { name, attributes, namespace } => {
                    let attributes = attributes.into_iter()
                                               .map(|o| Attribute::new(o.name.local_name, o.value))
                                               .collect();
                    let ns = if let Some(ref prefix) = name.prefix {
                        namespace.get(prefix)
                    }
                    else {
                        namespace.get(NS_NO_PREFIX)
                    }.map(|s| s.to_owned());
                    let mut root = Element::new(name.local_name, ns, attributes);
                    root.from_reader_inner(reader)?;
                    return Ok(root);
                },
                ReaderEvent::EndDocument => {
                    return Err(Error::EndOfDocument);
                },
                _ => () // TODO: may need more errors
            }
        }
    }

    fn from_reader_inner<R: Read>(&mut self, reader: &mut EventReader<R>) -> Result<(), Error> {
        loop {
            let e = reader.next()?;
            match e {
                ReaderEvent::StartElement { name, attributes, namespace } => {
                    let attributes = attributes.into_iter()
                                               .map(|o| Attribute::new(o.name.local_name, o.value))
                                               .collect();
                    let ns = if let Some(ref prefix) = name.prefix {
                        namespace.get(prefix)
                    }
                    else {
                        namespace.get(NS_NO_PREFIX)
                    }.map(|s| s.to_owned());
                    let elem = Element::new(name.local_name, ns, attributes);
                    let elem_ref = self.append_child(elem);
                    elem_ref.from_reader_inner(reader)?;
                },
                ReaderEvent::EndElement { .. } => {
                    // TODO: may want to check whether we're closing the correct element
                    return Ok(());
                },
                ReaderEvent::Characters(s) => {
                    self.append_text_node(s);
                },
                ReaderEvent::CData(s) => {
                    self.append_text_node(s);
                },
                ReaderEvent::EndDocument => {
                    return Err(Error::EndOfDocument);
                },
                _ => (), // TODO: may need to implement more
            }
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let name = if let Some(ref ns) = self.namespace {
            Name::qualified(&self.name, &ns, None)
        }
        else {
            Name::local(&self.name)
        };
        let mut start = WriterEvent::start_element(name);
        if let Some(ref ns) = self.namespace {
            start = start.default_ns(ns.as_ref());
        }
        for attr in &self.attributes { // TODO: I think this could be done a lot more efficiently
            start = start.attr(Name::local(&attr.name), &attr.value);
        }
        writer.write(start)?;
        for child in &self.children {
            match *child {
                Fork::Element(ref e) => {
                    e.write_to(writer)?;
                },
                Fork::Text(ref s) => {
                    writer.write(WriterEvent::characters(s))?;
                },
            }
        }
        writer.write(WriterEvent::end_element())?;
        Ok(())
    }

    pub fn children<'a>(&'a self) -> Children<'a> {
        Children {
            iter: self.children.iter(),
        }
    }

    pub fn children_mut<'a>(&'a mut self) -> ChildrenMut<'a> {
        ChildrenMut {
            iter: self.children.iter_mut(),
        }
    }

    pub fn append_child(&mut self, mut child: Element) -> &mut Element {
        if child.namespace.is_none() {
            child.namespace = self.namespace.clone();
        }
        self.children.push(Fork::Element(child));
        if let Fork::Element(ref mut cld) = *self.children.last_mut().unwrap() {
            cld
        }
        else {
            unreachable!()
        }
    }

    pub fn append_text_node<S: Into<String>>(&mut self, child: S) {
        self.children.push(Fork::Text(child.into()));
    }

    pub fn text(&self) -> String {
        let mut ret = String::new();
        for fork in &self.children {
            if let Fork::Text(ref s) = *fork {
                ret += s;
            }
        }
        ret
    }

    pub fn get_child<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> Option<&Element> {
        for fork in &self.children {
            if let Fork::Element(ref e) = *fork {
                if e.is(name.as_ref(), namespace.as_ref()) {
                    return Some(e);
                }
            }
        }
        None
    }

    pub fn get_child_mut<N: AsRef<str>, NS: AsRef<str>>(&mut self, name: N, namespace: NS) -> Option<&mut Element> {
        for fork in &mut self.children {
            if let Fork::Element(ref mut e) = *fork {
                if e.is(name.as_ref(), namespace.as_ref()) {
                    return Some(e);
                }
            }
        }
        None
    }

    pub fn has_child<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> bool {
        self.get_child(name, namespace).is_some()
    }
}

pub struct Children<'a> {
    iter: slice::Iter<'a, Fork>,
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a Element;

    fn next(&mut self) -> Option<&'a Element> {
        while let Some(item) = self.iter.next() {
            if let Fork::Element(ref child) = *item {
                return Some(child);
            }
        }
        None
    }
}

pub struct ChildrenMut<'a> {
    iter: slice::IterMut<'a, Fork>,
}

impl<'a> Iterator for ChildrenMut<'a> {
    type Item = &'a mut Element;

    fn next(&mut self) -> Option<&'a mut Element> {
        while let Some(item) = self.iter.next() {
            if let Fork::Element(ref mut child) = *item {
                return Some(child);
            }
        }
        None
    }
}

pub struct ElementBuilder {
    name: String,
    namespace: Option<String>,
    attributes: Vec<Attribute>,
}

impl ElementBuilder {
    pub fn ns<S: Into<String>>(mut self, namespace: S) -> ElementBuilder {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn attr<S: Into<String>, V: Into<String>>(mut self, name: S, value: V) -> ElementBuilder {
        self.attributes.push(Attribute::new(name, value));
        self
    }

    pub fn build(self) -> Element {
        Element::new(self.name, self.namespace, self.attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xml::reader::EventReader;
    use xml::writer::EventWriter;

    const TEST_STRING: &'static str = r#"<?xml version="1.0" encoding="utf-8"?><root xmlns="root_ns" a="b">meow<child c="d" /><child xmlns="child_ns" d="e" />nya</root>"#;

    fn build_test_tree() -> Element {
        let mut root = Element::builder("root")
                               .ns("root_ns")
                               .attr("a", "b")
                               .build();
        root.append_text_node("meow");
        let child = Element::builder("child")
                            .attr("c", "d")
                            .build();
        root.append_child(child);
        let other_child = Element::builder("child")
                                  .ns("child_ns")
                                  .attr("d", "e")
                                  .build();
        root.append_child(other_child);
        root.append_text_node("nya");
        root
    }

    #[test]
    fn reader_works() {
        use std::io::Cursor;
        let mut reader = EventReader::new(Cursor::new(TEST_STRING));
        assert_eq!(Element::from_reader(&mut reader).unwrap(), build_test_tree());
    }

    #[test]
    fn writer_works() {
        let root = build_test_tree();
        let mut out = Vec::new();
        {
            let mut writer = EventWriter::new(&mut out);
            root.write_to(&mut writer).unwrap();
        }
        assert_eq!(String::from_utf8(out).unwrap(), TEST_STRING);
    }

    #[test]
    fn builder_works() {
        let elem = Element::builder("a")
                           .ns("b")
                           .attr("c", "d")
                           .build();
        assert_eq!(elem.name(), "a");
        assert_eq!(elem.ns(), Some("b"));
        assert_eq!(elem.attr("c"), Some("d"));
        assert_eq!(elem.attr("x"), None);
        assert!(elem.is("a", "b"));
    }

    #[test]
    fn children_iter_works() {
        let root = build_test_tree();
        let mut iter = root.children();
        assert!(iter.next().unwrap().is("child", "root_ns"));
        assert!(iter.next().unwrap().is("child", "child_ns"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn get_child_works() {
        let root = build_test_tree();
        assert_eq!(root.get_child("child", "inexistent_ns"), None);
        assert_eq!(root.get_child("not_a_child", "root_ns"), None);
        assert!(root.get_child("child", "root_ns").unwrap().is("child", "root_ns"));
        assert!(root.get_child("child", "child_ns").unwrap().is("child", "child_ns"));
        assert_eq!(root.get_child("child", "root_ns").unwrap().attr("c"), Some("d"));
        assert_eq!(root.get_child("child", "child_ns").unwrap().attr("d"), Some("e"));
    }
}
