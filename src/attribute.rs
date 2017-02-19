use xml::escape::escape_str_attribute;

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl fmt::Display for Attribute {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}=\"{}\"", self.name, escape_str_attribute(&self.value))
    }
}

impl Attribute {
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Attribute {
        Attribute {
            name: name.into(),
            value: value.into(),
        }
    }
}
