//! Provides an `Element` type, which represents DOM nodes, and a builder to create them with.

use std::io:: Write;
use std::collections::{btree_map, BTreeMap};

use std::str;
use std::rc::Rc;
use std::borrow::Cow;

use error::{Error, ErrorKind, Result};

use quick_xml::reader::Reader as EventReader;
use quick_xml::events::{Event, BytesStart};

use std::io::BufRead;

use std::str::FromStr;

use std::slice;

use convert::{IntoElements, IntoAttributeValue, ElementEmitter};
use namespace_set::NamespaceSet;

/// Escape XML text
pub fn write_escaped<W: Write>(writer: &mut W, input: &str) -> Result<()> {
    for c in input.chars() {
        match c {
            '&' => write!(writer, "&amp;")?,
            '<' => write!(writer, "&lt;")?,
            '>' => write!(writer, "&gt;")?,
            '\'' => write!(writer, "&apos;")?,
            '"' => write!(writer, "&quot;")?,
            _ => write!(writer, "{}", c)?,
        }
    }

    Ok(())
}

/// A node in an element tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// An `Element`.
    Element(Element),
    /// A text node.
    Text(String),
}

impl Node {
    /// Turns this into an `Element` if possible, else returns None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Node;
    ///
    /// let elm = Node::Element("<meow />".parse().unwrap());
    /// let txt = Node::Text("meow".to_owned());
    ///
    /// assert_eq!(elm.as_element().unwrap().name(), "meow");
    /// assert_eq!(txt.as_element(), None);
    /// ```
    pub fn as_element(&self) -> Option<&Element> {
        match *self {
            Node::Element(ref e) => Some(e),
            Node::Text(_) => None,
        }
    }

    /// Turns this into a `String` if possible, else returns None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Node;
    ///
    /// let elm = Node::Element("<meow />".parse().unwrap());
    /// let txt = Node::Text("meow".to_owned());
    ///
    /// assert_eq!(elm.as_text(), None);
    /// assert_eq!(txt.as_text().unwrap(), "meow");
    /// ```
    pub fn as_text(&self) -> Option<&str> {
        match *self {
            Node::Element(_) => None,
            Node::Text(ref s) => Some(s),
        }
    }

    fn write_to_inner<W: Write>(&self, writer: &mut W) -> Result<()>{
        match *self {
            Node::Element(ref elmt) => elmt.write_to_inner(writer)?,
            Node::Text(ref s) => write_escaped(writer, s)?,
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// A struct representing a DOM Element.
pub struct Element {
    prefix: Option<String>,
    name: String,
    namespaces: Rc<NamespaceSet>,
    attributes: BTreeMap<String, String>,
    children: Vec<Node>,
}

impl<'a> From<&'a Element> for String {
    fn from(elem: &'a Element) -> String {
        let mut writer = Vec::new();
        elem.write_to(&mut writer).unwrap();
        String::from_utf8(writer).unwrap()
    }
}


impl FromStr for Element {
    type Err = Error;

    fn from_str(s: &str) -> Result<Element> {
        let mut reader = EventReader::from_str(s);
        Element::from_reader(&mut reader)
    }
}

impl Element {
    fn new<NS: Into<NamespaceSet>>(name: String, prefix: Option<String>, namespaces: NS, attributes: BTreeMap<String, String>, children: Vec<Node>) -> Element {
        Element {
            prefix, name,
            namespaces: Rc::new(namespaces.into()),
            attributes: attributes,
            children: children,
        }
    }

    /// Return a builder for an `Element` with the given `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem = Element::builder("name")
    ///                    .ns("namespace")
    ///                    .attr("name", "value")
    ///                    .append("inner")
    ///                    .build();
    ///
    /// assert_eq!(elem.name(), "name");
    /// assert_eq!(elem.ns(), Some("namespace".to_owned()));
    /// assert_eq!(elem.attr("name"), Some("value"));
    /// assert_eq!(elem.attr("inexistent"), None);
    /// assert_eq!(elem.text(), "inner");
    /// ```
    pub fn builder<S: AsRef<str>>(name: S) -> ElementBuilder {
        let (prefix, name) = split_element_name(name).unwrap();
        ElementBuilder {
            root: Element::new(name, prefix, None, BTreeMap::new(), Vec::new()),
            namespaces: Default::default(),
        }
    }

    /// Returns a bare minimum `Element` with this name.
    ///
    /// # Examples
    ///
    /// ```rust
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
            prefix: None,
            name: name.into(),
            namespaces: Rc::new(NamespaceSet::default()),
            attributes: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    /// Returns a reference to the name of this element.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the namespace of this element, if it has one, else `None`.
    pub fn ns(&self) -> Option<String> {
        self.namespaces.get(&self.prefix)
    }

    /// Returns a reference to the value of the given attribute, if it exists, else `None`.
    pub fn attr(&self, name: &str) -> Option<&str> {
        if let Some(value) = self.attributes.get(name) {
            return Some(value)
        }
        None
    }

    /// Returns an iterator over the attributes of this element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elm: Element = "<elem a=\"b\" />".parse().unwrap();
    ///
    /// let mut iter = elm.attrs();
    ///
    /// assert_eq!(iter.next().unwrap(), ("a", "b"));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn attrs(&self) -> Attrs {
        Attrs {
            iter: self.attributes.iter(),
        }
    }

    /// Returns an iterator over the attributes of this element, with the value being a mutable
    /// reference.
    pub fn attrs_mut(&mut self) -> AttrsMut {
        AttrsMut {
            iter: self.attributes.iter_mut(),
        }
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
    /// ```rust
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
        self.name == name.as_ref() &&
            self.has_ns(namespace)
    }

    /// Returns whether the element has the given namespace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem = Element::builder("name").ns("namespace").build();
    ///
    /// assert_eq!(elem.has_ns("namespace"), true);
    /// assert_eq!(elem.has_ns("wrong"), false);
    /// ```
    pub fn has_ns<NS: AsRef<str>>(&self, namespace: NS) -> bool {
        self.namespaces.has(&self.prefix, namespace)
    }

    /// Parse a document from an `EventReader`.
    pub fn from_reader<R: BufRead>(reader: &mut EventReader<R>) -> Result<Element> {
        let mut buf = Vec::new();
        let root: Element;

        loop {
            let e = reader.read_event(&mut buf)?;
            match e {
                Event::Empty(ref e) | Event::Start(ref e) => {
                    root = build_element(e)?; // FIXME: could be break build_element(e)? when break value is stable
                    break;
                },
                Event::Eof => {
                    bail!(ErrorKind::EndOfDocument);
                },
                _ => () // TODO: may need more errors
            }
        };

        let mut stack = vec![root];

        loop {
            match reader.read_event(&mut buf)? {
                Event::Empty(ref e) => {
                    let elem = build_element(e)?;
                    // Since there is no Event::End after, directly append it to the current node
                    stack.last_mut().unwrap().append_child(elem);
                },
                Event::Start(ref e) => {
                    let elem = build_element(e)?;
                    stack.push(elem);
                },
                Event::End(ref e) => {
                    if stack.len() <= 1 {
                        break;
                    }
                    let elem = stack.pop().unwrap();
                    if let Some(to) = stack.last_mut() {
                        if elem.name().as_bytes() != e.name() {
                            bail!(ErrorKind::InvalidElementClosed);
                        }
                        to.append_child(elem);
                    }
                },
                Event::Text(s) | Event::CData(s) => {
                    let text = s.unescape_and_decode(reader)?;
                    if text != "" {
                        let mut current_elem = stack.last_mut().unwrap();
                        current_elem.append_text_node(text);
                    }
                },
                Event::Eof => {
                    break;
                },
                _ => (), // TODO: may need to implement more
            }
        }
        Ok(stack.pop().unwrap())
    }

    /// Output a document to a `Writer`.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        write!(writer, "<?xml version=\"1.0\" encoding=\"utf-8\"?>")?;
        self.write_to_inner(writer)
    }

    /// Like `write_to()` but without the `<?xml?>` prelude
    pub fn write_to_inner<W: Write>(&self, writer: &mut W) -> Result<()> {
        let name = match &self.prefix {
            &None => Cow::Borrowed(&self.name),
            &Some(ref prefix) => Cow::Owned(format!("{}:{}", prefix, self.name)),
        };
        write!(writer, "<{}", name)?;

        for (prefix, ns) in self.namespaces.declared_ns() {
            match prefix {
                &None => {
                    write!(writer, " xmlns=\"")?;
                    write_escaped(writer, ns)?;
                    write!(writer, "\"")?;
                },
                &Some(ref prefix) => {
                    write!(writer, " xmlns:{}=\"", prefix)?;
                    write_escaped(writer, ns)?;
                    write!(writer, "\"")?;
                },
            }
        }

        for (key, value) in &self.attributes {
            write!(writer, " {}=\"", key)?;
                write_escaped(writer, value)?;
                write!(writer, "\"")?;
        }

        if self.children.is_empty() {
            write!(writer, " />")?;
            return Ok(())
        }

        write!(writer, ">")?;

        for child in &self.children {
            child.write_to_inner(writer)?;
        }

        write!(writer, "</{}>", name)?;
        Ok(())
    }

    /// Returns an iterator over references to every child node of this element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem: Element = "<root>a<c1 />b<c2 />c</root>".parse().unwrap();
    ///
    /// let mut iter = elem.nodes();
    ///
    /// assert_eq!(iter.next().unwrap().as_text().unwrap(), "a");
    /// assert_eq!(iter.next().unwrap().as_element().unwrap().name(), "c1");
    /// assert_eq!(iter.next().unwrap().as_text().unwrap(), "b");
    /// assert_eq!(iter.next().unwrap().as_element().unwrap().name(), "c2");
    /// assert_eq!(iter.next().unwrap().as_text().unwrap(), "c");
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline] pub fn nodes(&self) -> Nodes {
        self.children.iter()
    }

    /// Returns an iterator over mutable references to every child node of this element.
    #[inline] pub fn nodes_mut(&mut self) -> NodesMut {
        self.children.iter_mut()
    }

    /// Returns an iterator over references to every child element of this element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem: Element = "<root>hello<child1 />this<child2 />is<child3 />ignored</root>".parse().unwrap();
    ///
    /// let mut iter = elem.children();
    /// assert_eq!(iter.next().unwrap().name(), "child1");
    /// assert_eq!(iter.next().unwrap().name(), "child2");
    /// assert_eq!(iter.next().unwrap().name(), "child3");
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline] pub fn children(&self) -> Children {
        Children {
            iter: self.children.iter(),
        }
    }

    /// Returns an iterator over mutable references to every child element of this element.
    #[inline] pub fn children_mut(&mut self) -> ChildrenMut {
        ChildrenMut {
            iter: self.children.iter_mut(),
        }
    }

    /// Returns an iterator over references to every text node of this element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem: Element = "<root>hello<c /> world!</root>".parse().unwrap();
    ///
    /// let mut iter = elem.texts();
    /// assert_eq!(iter.next().unwrap(), "hello");
    /// assert_eq!(iter.next().unwrap(), " world!");
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline] pub fn texts(&self) -> Texts {
        Texts {
            iter: self.children.iter(),
        }
    }

    /// Returns an iterator over mutable references to every text node of this element.
    #[inline] pub fn texts_mut(&mut self) -> TextsMut {
        TextsMut {
            iter: self.children.iter_mut(),
        }
    }

    /// Appends a child node to the `Element`, returning the appended node.
    ///
    /// # Examples
    ///
    /// ```rust
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
    pub fn append_child(&mut self, child: Element) -> &mut Element {
        child.namespaces.set_parent(self.namespaces.clone());

        self.children.push(Node::Element(child));
        if let Node::Element(ref mut cld) = *self.children.last_mut().unwrap() {
            cld
        } else {
            unreachable!()
        }
    }

    /// Appends a text node to an `Element`.
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// ```rust
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
    /// ```rust
    /// use minidom::Element;
    ///
    /// let elem: Element = "<node>hello,<split /> world!</node>".parse().unwrap();
    ///
    /// assert_eq!(elem.text(), "hello, world!");
    /// ```
    pub fn text(&self) -> String {
        self.texts().fold(String::new(), |ret, new| ret + new)
    }

    /// Returns a reference to the first child element with the specific name and namespace, if it
    /// exists in the direct descendants of this `Element`, else returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// ```rust
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

fn split_element_name<S: AsRef<str>>(s: S) -> Result<(Option<String>, String)> {
    let name_parts = s.as_ref().split(':').collect::<Vec<&str>>();
    match name_parts.len() {
        2 => Ok((Some(name_parts[0].to_owned()), name_parts[1].to_owned())),
        1 => Ok((None, name_parts[0].to_owned())),
        _ => bail!(ErrorKind::InvalidElement),
    }
}

fn build_element(event: &BytesStart) -> Result<Element> {
    let mut namespaces = BTreeMap::new();
    let attributes = event.attributes()
        .map(|o| {
            let o = o?;
            let key = str::from_utf8(o.key)?.to_owned();
            let value = str::from_utf8(o.value)?.to_owned();
            Ok((key, value))
        })
        .filter(|o| {
            match o {
                &Ok((ref key, ref value)) if key == "xmlns" => {
                    namespaces.insert(None, value.to_owned());
                    false
                },
                &Ok((ref key, ref value)) if key.starts_with("xmlns:") => {
                    namespaces.insert(Some(key[6..].to_owned()), value.to_owned());
                    false
                },
                _ => true,
            }
        })
        .collect::<Result<BTreeMap<String, String>>>()?;

    let (prefix, name) = split_element_name(str::from_utf8(event.name())?)?;
    let element = Element::new(name, prefix, namespaces, attributes, Vec::new());
    Ok(element)
}

/// An iterator over references to child elements of an `Element`.
pub struct Children<'a> {
    iter: slice::Iter<'a, Node>,
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a Element;

    fn next(&mut self) -> Option<&'a Element> {
        for item in &mut self.iter {
            if let Node::Element(ref child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// An iterator over mutable references to child elements of an `Element`.
pub struct ChildrenMut<'a> {
    iter: slice::IterMut<'a, Node>,
}

impl<'a> Iterator for ChildrenMut<'a> {
    type Item = &'a mut Element;

    fn next(&mut self) -> Option<&'a mut Element> {
        for item in &mut self.iter {
            if let Node::Element(ref mut child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// An iterator over references to child text nodes of an `Element`.
pub struct Texts<'a> {
    iter: slice::Iter<'a, Node>,
}

impl<'a> Iterator for Texts<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        for item in &mut self.iter {
            if let Node::Text(ref child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// An iterator over mutable references to child text nodes of an `Element`.
pub struct TextsMut<'a> {
    iter: slice::IterMut<'a, Node>,
}

impl<'a> Iterator for TextsMut<'a> {
    type Item = &'a mut String;

    fn next(&mut self) -> Option<&'a mut String> {
        for item in &mut self.iter {
            if let Node::Text(ref mut child) = *item {
                return Some(child);
            }
        }
        None
    }
}

/// An iterator over references to all child nodes of an `Element`.
pub type Nodes<'a> = slice::Iter<'a, Node>;

/// An iterator over mutable references to all child nodes of an `Element`.
pub type NodesMut<'a> = slice::IterMut<'a, Node>;

/// An iterator over the attributes of an `Element`.
pub struct Attrs<'a> {
    iter: btree_map::Iter<'a, String, String>,
}

impl<'a> Iterator for Attrs<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(x, y)| (x.as_ref(), y.as_ref()))
    }
}

/// An iterator over the attributes of an `Element`, with the values mutable.
pub struct AttrsMut<'a> {
    iter: btree_map::IterMut<'a, String, String>,
}

impl<'a> Iterator for AttrsMut<'a> {
    type Item = (&'a str, &'a mut String);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(x, y)| (x.as_ref(), y))
    }
}

/// A builder for `Element`s.
pub struct ElementBuilder {
    root: Element,
    namespaces: BTreeMap<Option<String>, String>,
}

impl ElementBuilder {
    /// Sets the namespace.
    pub fn ns<S: Into<String>>(mut self, namespace: S) -> ElementBuilder {
        self.namespaces
            .insert(self.root.prefix.clone(), namespace.into());
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
        let mut element = self.root;
        // Set namespaces
        element.namespaces = Rc::new(NamespaceSet::from(self.namespaces));
        // Propagate namespaces
        for node in &element.children {
            if let Node::Element(ref e) = *node {
                e.namespaces.set_parent(element.namespaces.clone());
            }
        }

        element
    }
}

#[cfg(test)]
#[test]
fn test_element_new() {
    use std::iter::FromIterator;

    let elem = Element::new( "name".to_owned()
                           , None
                           , Some("namespace".to_owned())
                           , BTreeMap::from_iter(vec![ ("name".to_string(), "value".to_string()) ].into_iter() )
                           , Vec::new() );

    assert_eq!(elem.name(), "name");
    assert_eq!(elem.ns(), Some("namespace".to_owned()));
    assert_eq!(elem.attr("name"), Some("value"));
    assert_eq!(elem.attr("inexistent"), None);
}
