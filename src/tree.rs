// TODO: really should either be a separate crate or implemented into xml-rs

use std::io::prelude::*;

use std::convert::From;

use std::fmt;

use xml::name::{OwnedName, Name};
use xml::reader::{XmlEvent as ReaderEvent, EventReader};
use xml::writer::{XmlEvent as WriterEvent, EventWriter};
use xml::attribute::OwnedAttribute;

use error::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Element {
    name: OwnedName,
    attributes: Vec<OwnedAttribute>,
    children: Vec<Fork>,
}

impl fmt::Debug for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "<{}", self.name)?;
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
    pub fn new(name: OwnedName, attributes: Vec<OwnedAttribute>) -> Element {
        Element {
            name: name,
            attributes: attributes,
            children: Vec::new(),
        }
    }

    pub fn builder<S: Into<String>>(name: S) -> ElementBuilder {
        ElementBuilder {
            name: OwnedName::local(name),
            attributes: Vec::new(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.name.local_name
    }

    pub fn ns(&self) -> Option<&str> {
        self.name.namespace.as_ref()
                           .map(String::as_ref)
    }

    pub fn attr(&self, key: &str) -> Option<&str> {
        for attr in &self.attributes {
            if attr.name.local_name == key {
                return Some(&attr.value);
            }
        }
        None
    }

    pub fn from_reader<R: Read>(reader: &mut EventReader<R>) -> Result<Element, Error> {
        loop {
            let e = reader.next()?;
            match e {
                ReaderEvent::StartElement { name, attributes, .. } => {
                    let mut root = Element::new(name, attributes);
                    root.from_reader_inner(reader);
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
                ReaderEvent::StartElement { name, attributes, .. } => {
                    let elem = Element::new(name, attributes);
                    let elem_ref = self.append_child(elem);
                    elem_ref.from_reader_inner(reader);
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
        let mut start = WriterEvent::start_element(self.name.borrow());
        if let Some(ref ns) = self.name.namespace {
            start = start.default_ns(ns.as_ref());
        }
        for attr in &self.attributes { // TODO: I think this could be done a lot more efficiently
            start = start.attr(attr.name.borrow(), &attr.value);
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
        unimplemented!();
    }

    pub fn children_mut<'a>(&'a mut self) -> ChildrenMut<'a> {
        unimplemented!();
    }

    pub fn append_child(&mut self, child: Element) -> &mut Element {
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

    pub fn text(&self) -> &str {
        unimplemented!()
    }

    pub fn get_child<'a, N: Into<Name<'a>>>(&self, name: N) -> Option<&Element> {
        unimplemented!()
    }

    pub fn get_child_mut<'a, N: Into<Name<'a>>>(&mut self, name: N) -> Option<&mut Element> {
        unimplemented!()
    }

    pub fn into_child<'a, N: Into<Name<'a>>>(self, name: N) -> Option<Element> {
        unimplemented!()
    }
}

pub struct Children<'a> {
    elem: &'a Element,
}

pub struct ChildrenMut<'a> {
    elem: &'a mut Element,
}

pub struct ElementBuilder {
    name: OwnedName,
    attributes: Vec<OwnedAttribute>,
}

impl ElementBuilder {
    pub fn ns<S: Into<String>>(mut self, namespace: S) -> ElementBuilder {
        self.name.namespace = Some(namespace.into());
        self
    }

    pub fn attr<S: Into<String>, V: Into<String>>(mut self, name: S, value: V) -> ElementBuilder {
        self.attributes.push(OwnedAttribute::new(OwnedName::local(name), value));
        self
    }

    pub fn attr_ns<S: Into<String>, N: Into<String>, V: Into<String>>(mut self, name: S, namespace: N, value: V) -> ElementBuilder {
        self.attributes.push(OwnedAttribute::new(OwnedName::qualified::<_, _, &'static str>(name, namespace, None), value));
        self
    }

    pub fn build(self) -> Element {
        Element::new(self.name, self.attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // TODO: fix a bunch of namespace stuff so this test passes
        assert_eq!(Element::from_reader(&mut reader).unwrap(), build_test_tree());
    }

    #[test]
    fn writer_works() {
        let root = build_test_tree();
        let mut out = Vec::new();
        {
            let mut writer = EventWriter::new(&mut out);
            root.write_to(&mut writer);
        }
        assert_eq!(String::from_utf8(out).unwrap(), TEST_STRING);
    }

    #[test]
    fn builder_works() {
        let elem = Element::builder("a")
                           .ns("b")
                           .attr("c", "d")
                           .build();
        assert_eq!(elem.tag(), "a");
        assert_eq!(elem.ns(), Some("b"));
        assert_eq!(elem.attr("c"), Some("d"));
    }
}
