//! Provides an `Element` type, which represents DOM nodes, and a builder to create them with.

use std::io::prelude::*;
use std::io::Cursor;
use std::collections::BTreeMap;
use std::iter::FromIterator;

use std::fmt;

use error::Error;

use xml::reader::{XmlEvent as ReaderEvent, EventReader};
use xml::writer::{XmlEvent as WriterEvent, EventWriter, EmitterConfig};
use xml::name::Name;
use xml::namespace::NS_NO_PREFIX;

use std::str::FromStr;

use std::slice;

use convert::{IntoElements, IntoAttributeValue, ElementEmitter};

#[derive(Clone, PartialEq, Eq)]
/// A struct representing a DOM Element.
pub struct Element {
    name: String,
    namespace: Option<String>,
    attributes: BTreeMap<String, String>,
    children: Vec<Node>,
}


impl<'a> From<&'a Element> for String {
    fn from(elem: &'a Element) -> String {
        let mut out = Vec::new();
        let config = EmitterConfig::new()
                    .write_document_declaration(false);
        elem.write_to(&mut EventWriter::new_with_config(&mut out, config)).unwrap();
        String::from_utf8(out).unwrap()
    }
}

impl fmt::Debug for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", String::from(self))?;
        Ok(())
    }
}

impl FromStr for Element {
    type Err = Error;

    fn from_str(s: &str) -> Result<Element, Error> {
        let mut reader = EventReader::new(Cursor::new(s));
        Element::from_reader(&mut reader)
    }
}

/// A node in an element tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// An `Element`.
    Element(Element),
    /// A text node.
    Text(String),
}

impl Element {
    fn new(name: String, namespace: Option<String>, attributes: BTreeMap<String, String>, children: Vec<Node>) -> Element {
        Element {
            name: name,
            namespace: namespace,
            attributes: attributes,
            children: children,
        }
    }

    /// Return a builder for an `Element` with the given `name`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem = Element::builder("name")
    ///                    .ns("namespace")
    ///                    .attr("name", "value")
    ///                    .append("inner")
    ///                    .build();
    ///
    /// assert_eq!(elem.name(), "name");
    /// assert_eq!(elem.ns(), Some("namespace"));
    /// assert_eq!(elem.attr("name"), Some("value"));
    /// assert_eq!(elem.attr("inexistent"), None);
    /// assert_eq!(elem.text(), "inner");
    /// ```
    pub fn builder<S: Into<String>>(name: S) -> ElementBuilder {
        ElementBuilder {
            root: Element::new(name.into(), None, BTreeMap::new(), Vec::new()),
        }
    }

    /// Returns a bare minimum `Element` with this name.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let bare = Element::bare("name");
    ///
    /// assert_eq!(bare.name(), "name");
    /// assert_eq!(bare.ns(), None);
    /// assert_eq!(bare.attr("name"), None);
    /// assert_eq!(bare.text(), "");
    /// ```
    pub fn bare<S: Into<String>>(name: S) -> Element {
        Element {
            name: name.into(),
            namespace: None,
            attributes: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    /// Returns a reference to the name of this element.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the namespace of this element, if it has one, else `None`.
    pub fn ns(&self) -> Option<&str> {
        self.namespace.as_ref()
                      .map(String::as_ref)
    }

    /// Returns a reference to the value of the given attribute, if it exists, else `None`.
    pub fn attr(&self, name: &str) -> Option<&str> {
        if let Some(value) = self.attributes.get(name) {
            return Some(&value)
        }
        None
    }

    /// Modifies the value of an attribute.
    pub fn set_attr<S: Into<String>, V: IntoAttributeValue>(&mut self, name: S, val: V) {
        let name = name.into();
        let val = val.into_attribute_value();

        if let Some(value) = self.attributes.get_mut(&name) {
            *value = val.expect("removing existing value via set_attr, this is not yet supported (TODO)"); // TODO
            return;
        }

        if let Some(val) = val {
            self.attributes.insert(name, val);
        }
    }

    /// Returns whether the element has the given name and namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem = Element::builder("name").ns("namespace").build();
    ///
    /// assert_eq!(elem.is("name", "namespace"), true);
    /// assert_eq!(elem.is("name", "wrong"), false);
    /// assert_eq!(elem.is("wrong", "namespace"), false);
    /// assert_eq!(elem.is("wrong", "wrong"), false);
    /// ```
    pub fn is<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> bool {
        let ns = self.namespace.as_ref().map(String::as_ref);
        self.name == name.as_ref() && ns == Some(namespace.as_ref())
    }

    /// Parse a document from an `EventReader`.
    pub fn from_reader<R: Read>(reader: &mut EventReader<R>) -> Result<Element, Error> {
        loop {
            let e = reader.next()?;
            match e {
                ReaderEvent::StartElement { name, attributes, namespace } => {
                    let attributes = attributes.into_iter()
                                               .map(|o| {
                                                    (match o.name.prefix {
                                                        Some(prefix) => format!("{}:{}", prefix, o.name.local_name),
                                                        None => o.name.local_name
                                                    },
                                                    o.value)
                                                })
                                               .collect();
                    let ns = if let Some(ref prefix) = name.prefix {
                        namespace.get(prefix)
                    }
                    else {
                        namespace.get(NS_NO_PREFIX)
                    }.map(|s| s.to_owned());

                    let mut root = Element::new(name.local_name, ns, attributes, Vec::new());
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
                                               .map(|o| {
                                                    (match o.name.prefix {
                                                        Some(prefix) => format!("{}:{}", prefix, o.name.local_name),
                                                        None => o.name.local_name
                                                    },
                                                    o.value)
                                                })
                                               .collect();
                    let ns = if let Some(ref prefix) = name.prefix {
                        namespace.get(prefix)
                    }
                    else {
                        namespace.get(NS_NO_PREFIX)
                    }.map(|s| s.to_owned());
                    let elem = Element::new(name.local_name, ns, attributes, Vec::with_capacity(1));
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

    /// Output a document to an `EventWriter`.
    pub fn write_to<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let name = if let Some(ref ns) = self.namespace {
            Name::qualified(&self.name, &ns, None)
        }
        else {
            Name::local(&self.name)
        };
        let mut start = WriterEvent::start_element(name);
        if let Some(ref ns) = self.namespace {
            start = start.default_ns(ns.clone());
        }
        for attr in &self.attributes { // TODO: I think this could be done a lot more efficiently
            start = start.attr(Name::local(&attr.0), &attr.1);
        }
        writer.write(start)?;
        for child in &self.children {
            match *child {
                Node::Element(ref e) => {
                    e.write_to(writer)?;
                },
                Node::Text(ref s) => {
                    writer.write(WriterEvent::characters(s))?;
                },
            }
        }
        writer.write(WriterEvent::end_element())?;
        Ok(())
    }

    /// Returns an iterator over references to the children of this element.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem: Element = "<root><child1 /><child2 /><child3 /></root>".parse().unwrap();
    ///
    /// let mut iter = elem.children();
    /// assert_eq!(iter.next().unwrap().name(), "child1");
    /// assert_eq!(iter.next().unwrap().name(), "child2");
    /// assert_eq!(iter.next().unwrap().name(), "child3");
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn children<'a>(&'a self) -> Children<'a> {
        Children {
            iter: self.children.iter(),
        }
    }

    /// Returns an iterator over mutable references to the children of this element.
    pub fn children_mut<'a>(&'a mut self) -> ChildrenMut<'a> {
        ChildrenMut {
            iter: self.children.iter_mut(),
        }
    }

    fn propagate_namespaces(&mut self) {
        let ns = self.namespace.clone();
        for child in self.children_mut() {
            if child.namespace.is_none() {
                child.namespace = ns.clone();
                child.propagate_namespaces();
            }
        }
    }

    /// Appends a child node to the `Element`, returning the appended node.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let mut elem = Element::bare("root");
    ///
    /// assert_eq!(elem.children().count(), 0);
    ///
    /// elem.append_child(Element::bare("child"));
    ///
    /// {
    ///     let mut iter = elem.children();
    ///     assert_eq!(iter.next().unwrap().name(), "child");
    ///     assert_eq!(iter.next(), None);
    /// }
    ///
    /// let child = elem.append_child(Element::bare("new"));
    ///
    /// assert_eq!(child.name(), "new");
    /// ```
    pub fn append_child(&mut self, mut child: Element) -> &mut Element {
        if child.namespace.is_none() && self.namespace.is_some() {
            child.namespace = self.namespace.clone();
            child.propagate_namespaces();
        }
        self.children.push(Node::Element(child));
        if let Node::Element(ref mut cld) = *self.children.last_mut().unwrap() {
            cld
        }
        else {
            unreachable!()
        }
    }

    /// Appends a text node to an `Element`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let mut elem = Element::bare("node");
    ///
    /// assert_eq!(elem.text(), "");
    ///
    /// elem.append_text_node("text");
    ///
    /// assert_eq!(elem.text(), "text");
    /// ```
    pub fn append_text_node<S: Into<String>>(&mut self, child: S) {
        self.children.push(Node::Text(child.into()));
    }

    /// Appends a node to an `Element`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::{Element, Node};
    ///
    /// let mut elem = Element::bare("node");
    ///
    /// elem.append_node(Node::Text("hello".to_owned()));
    ///
    /// assert_eq!(elem.text(), "hello");
    /// ```
    pub fn append_node(&mut self, node: Node) {
        self.children.push(node);
    }

    /// Returns the concatenation of all text nodes in the `Element`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem: Element = "<node>hello, world!</node>".parse().unwrap();
    ///
    /// assert_eq!(elem.text(), "hello, world!");
    /// ```
    pub fn text(&self) -> String {
        let mut ret = String::new();
        for fork in &self.children {
            if let Node::Text(ref s) = *fork {
                ret += s;
            }
        }
        ret
    }

    /// Returns a reference to the first child element with the specific name and namespace, if it
    /// exists in the direct descendants of this `Element`, else returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem: Element = r#"<node xmlns="ns"><a /><a xmlns="other_ns" /><b /></node>"#.parse().unwrap();
    ///
    /// assert!(elem.get_child("a", "ns").unwrap().is("a", "ns"));
    /// assert!(elem.get_child("a", "other_ns").unwrap().is("a", "other_ns"));
    /// assert!(elem.get_child("b", "ns").unwrap().is("b", "ns"));
    /// assert_eq!(elem.get_child("c", "ns"), None);
    /// assert_eq!(elem.get_child("b", "other_ns"), None);
    /// assert_eq!(elem.get_child("a", "inexistent_ns"), None);
    /// ```
    pub fn get_child<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> Option<&Element> {
        for fork in &self.children {
            if let Node::Element(ref e) = *fork {
                if e.is(name.as_ref(), namespace.as_ref()) {
                    return Some(e);
                }
            }
        }
        None
    }

    /// Returns a mutable reference to the first child element with the specific name and namespace,
    /// if it exists in the direct descendants of this `Element`, else returns `None`.
    pub fn get_child_mut<N: AsRef<str>, NS: AsRef<str>>(&mut self, name: N, namespace: NS) -> Option<&mut Element> {
        for fork in &mut self.children {
            if let Node::Element(ref mut e) = *fork {
                if e.is(name.as_ref(), namespace.as_ref()) {
                    return Some(e);
                }
            }
        }
        None
    }

    /// Returns whether a specific child with this name and namespace exists in the direct
    /// descendants of the `Element`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Element;
    ///
    /// let elem: Element = r#"<node xmlns="ns"><a /><a xmlns="other_ns" /><b /></node>"#.parse().unwrap();
    ///
    /// assert_eq!(elem.has_child("a", "other_ns"), true);
    /// assert_eq!(elem.has_child("a", "ns"), true);
    /// assert_eq!(elem.has_child("a", "inexistent_ns"), false);
    /// assert_eq!(elem.has_child("b", "ns"), true);
    /// assert_eq!(elem.has_child("b", "other_ns"), false);
    /// assert_eq!(elem.has_child("b", "inexistent_ns"), false);
    /// ```
    pub fn has_child<N: AsRef<str>, NS: AsRef<str>>(&self, name: N, namespace: NS) -> bool {
        self.get_child(name, namespace).is_some()
    }
}

/// An iterator over references to children of an `Element`.
pub struct Children<'a> {
    iter: slice::Iter<'a, Node>,
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a Element;

    fn next(&mut self) -> Option<&'a Element> {
        while let Some(item) = self.iter.next() {
            if let Node::Element(ref child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// An iterator over mutable references to children of an `Element`.
pub struct ChildrenMut<'a> {
    iter: slice::IterMut<'a, Node>,
}

impl<'a> Iterator for ChildrenMut<'a> {
    type Item = &'a mut Element;

    fn next(&mut self) -> Option<&'a mut Element> {
        while let Some(item) = self.iter.next() {
            if let Node::Element(ref mut child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// A builder for `Element`s.
pub struct ElementBuilder {
    root: Element,
}

impl ElementBuilder {
    /// Sets the namespace.
    pub fn ns<S: Into<String>>(mut self, namespace: S) -> ElementBuilder {
        self.root.namespace = Some(namespace.into());
        self
    }

    /// Sets an attribute.
    pub fn attr<S: Into<String>, V: IntoAttributeValue>(mut self, name: S, value: V) -> ElementBuilder {
        self.root.set_attr(name, value);
        self
    }

    /// Appends anything implementing `IntoElements` into the tree.
    pub fn append<T: IntoElements>(mut self, into: T) -> ElementBuilder {
        {
            let mut emitter = ElementEmitter::new(&mut self.root);
            into.into_elements(&mut emitter);
        }
        self
    }

    /// Builds the `Element`.
    pub fn build(self) -> Element {
        self.root
    }
}

#[test]
fn test_element_new() {
    let elem = Element::new( "name".to_owned()
                           , Some("namespace".to_owned())
                           , BTreeMap::from_iter(vec![ ("name".to_string(), "value".to_string()) ].into_iter() )
                           , Vec::new() );

    assert_eq!(elem.name(), "name");
    assert_eq!(elem.ns(), Some("namespace"));
    assert_eq!(elem.attr("name"), Some("value"));
    assert_eq!(elem.attr("inexistent"), None);
}
