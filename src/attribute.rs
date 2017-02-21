use xml::escape::escape_str_attribute;

use std::fmt;

/// An attribute of a DOM element.
///
/// This is of the form: `name`="`value`"
///
/// This does not support prefixed/namespaced attributes yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The value of the attribute.
    pub value: String,
}

impl fmt::Display for Attribute {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}=\"{}\"", self.name, escape_str_attribute(&self.value))
    }
}

impl Attribute {
    /// Construct a new attribute from the given `name` and `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minidom::Attribute;
    ///
    /// let attr = Attribute::new("name", "value");
    ///
    /// assert_eq!(attr.name, "name");
    /// assert_eq!(attr.value, "value");
    /// ```
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Attribute {
        Attribute {
            name: name.into(),
            value: value.into(),
        }
    }
}
