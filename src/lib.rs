//! A minimal DOM crate built on top of xml-rs.
//!
//! This library exports an `Element` struct which represents a DOM tree.
//!
//! # Example
//!
//! Run with `cargo run --example articles`. Located in `examples/articles.rs`.
//!
//! ```rust,ignore
//! extern crate minidom;
//!
//! use minidom::Element;
//!
//! const DATA: &'static str = r#"<articles xmlns="article">
//!     <article>
//!         <title>10 Terrible Bugs You Would NEVER Believe Happened</title>
//!         <body>
//!             Rust fixed them all. &lt;3
//!         </body>
//!     </article>
//!     <article>
//!         <title>BREAKING NEWS: Physical Bug Jumps Out Of Programmer's Screen</title>
//!         <body>
//!             Just kidding!
//!         </body>
//!     </article>
//! </articles>"#;
//!
//! const ARTICLE_NS: &'static str = "article";
//!
//! #[derive(Debug)]
//! pub struct Article {
//!     title: String,
//!     body: String,
//! }
//!
//! fn main() {
//!     let root: Element = DATA.parse().unwrap();
//!
//!     let mut articles: Vec<Article> = Vec::new();
//!
//!     for child in root.children() {
//!         if child.is("article", ARTICLE_NS) {
//!             let title = child.get_child("title", ARTICLE_NS).unwrap().text();
//!             let body = child.get_child("body", ARTICLE_NS).unwrap().text();
//!             articles.push(Article {
//!                 title: title,
//!                 body: body.trim().to_owned(),
//!             });
//!         }
//!     }
//!
//!     println!("{:?}", articles);
//! }
//! ```
//!
//! # Usage
//!
//! To use `minidom`, add this to your `Cargo.toml`:
//!
//! ```toml,ignore
//! [dependencies.minidom]
//! git = "https://gitlab.com/lumi/minidom-rs.git"
//! ```

extern crate xml;

mod error;

mod attribute;

use std::io::prelude::*;
use std::io::Cursor;

use std::convert::AsRef;

use std::iter::Iterator;

use std::slice;

use std::fmt;

use std::str::FromStr;

use xml::reader::{XmlEvent as ReaderEvent, EventReader};
use xml::writer::{XmlEvent as WriterEvent, EventWriter};
use xml::name::Name;
use xml::namespace::NS_NO_PREFIX;

pub use error::Error;

pub use attribute::Attribute;

#[derive(Clone, PartialEq, Eq)]
/// A struct representing a DOM Element.
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

impl FromStr for Element {
    type Err = Error;

    fn from_str(s: &str) -> Result<Element, Error> {
        let mut reader = EventReader::new(Cursor::new(s));
        Element::from_reader(&mut reader)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Fork {
    Element(Element),
    Text(String),
}

impl Element {
    /// Constructs a new `Element` with the given `name`, `namespace` and `attributes`.
    ///
    /// You probably should be using `Element::builder` instead of this.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::{Element, Attribute};
    ///
    /// let elem = Element::new( "name".to_owned()
    ///                        , Some("namespace".to_owned())
    ///                        , vec![ Attribute::new("name", "value") ] );
    ///
    /// assert_eq!(elem.name(), "name");
    /// assert_eq!(elem.ns(), Some("namespace"));
    /// assert_eq!(elem.attr("name"), Some("value"));
    /// assert_eq!(elem.attr("inexistent"), None);
    /// ```
    pub fn new(name: String, namespace: Option<String>, attributes: Vec<Attribute>) -> Element {
        Element {
            name: name,
            namespace: namespace,
            attributes: attributes,
            children: Vec::new(),
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
    ///                    .text("inner")
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
            name: name.into(),
            text: None,
            namespace: None,
            attributes: Vec::new(),
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
            attributes: Vec::new(),
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
        for attr in &self.attributes {
            if attr.name == name {
                return Some(&attr.value);
            }
        }
        None
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
        self.children.push(Fork::Text(child.into()));
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
            if let Fork::Text(ref s) = *fork {
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
            if let Fork::Element(ref e) = *fork {
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
            if let Fork::Element(ref mut e) = *fork {
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

/// An iterator over mutable references to children of an `Element`.
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

/// A builder for `Element`s.
pub struct ElementBuilder {
    name: String,
    text: Option<String>,
    namespace: Option<String>,
    attributes: Vec<Attribute>,
}

impl ElementBuilder {
    /// Sets the namespace.
    pub fn ns<S: Into<String>>(mut self, namespace: S) -> ElementBuilder {
        self.namespace = Some(namespace.into());
        self
    }

    /// Sets an attribute.
    pub fn attr<S: Into<String>, V: Into<String>>(mut self, name: S, value: V) -> ElementBuilder {
        self.attributes.push(Attribute::new(name, value));
        self
    }

    /// Sets the inner text.
    pub fn text<S: Into<String>>(mut self, text: S) -> ElementBuilder {
        self.text = Some(text.into());
        self
    }

    /// Builds the `Element`.
    pub fn build(self) -> Element {
        let mut elem = Element::new(self.name, self.namespace, self.attributes);
        if let Some(text) = self.text {
            elem.append_text_node(text);
        }
        elem
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
                           .text("e")
                           .build();
        assert_eq!(elem.name(), "a");
        assert_eq!(elem.ns(), Some("b"));
        assert_eq!(elem.attr("c"), Some("d"));
        assert_eq!(elem.attr("x"), None);
        assert_eq!(elem.text(), "e");
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
