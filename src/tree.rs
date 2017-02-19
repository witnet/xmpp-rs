// TODO: really should either be a separate crate or implemented into xml-rs

use std::io::prelude::*;

use std::fmt;

use xml::name::{OwnedName, Name};
use xml::reader::{XmlEvent, EventReader};
use xml::writer::{XmlEvent as WriterEvent, EventWriter};
use xml::attribute::OwnedAttribute;

use error::Error;

#[derive(Clone)]
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

#[derive(Clone, Debug)]
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

    pub fn from_reader<R: Read>(reader: &mut EventReader<R>) -> Result<Element, Error> {
        loop {
            let e = reader.next()?;
            match e {
                XmlEvent::StartElement { name, attributes, .. } => {
                    let mut root = Element::new(name, attributes);
                    root.from_reader_inner(reader);
                    return Ok(root);
                },
                XmlEvent::EndDocument => {
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
                XmlEvent::StartElement { name, attributes, .. } => {
                    let elem = Element::new(name, attributes);
                    let elem_ref = self.append_child(elem);
                    elem_ref.from_reader_inner(reader);
                },
                XmlEvent::EndElement { .. } => {
                    // TODO: may want to check whether we're closing the correct element
                    return Ok(());
                },
                XmlEvent::Characters(s) => {
                    self.append_text_node(s);
                },
                XmlEvent::CData(s) => {
                    self.append_text_node(s);
                },
                XmlEvent::EndDocument => {
                    return Err(Error::EndOfDocument);
                },
                _ => (), // TODO: may need to implement more
            }
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), Error> {
        let start = WriterEvent::start_element(self.name.borrow());
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

    pub fn append_text_node(&mut self, child: String) {
        self.children.push(Fork::Text(child));
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
