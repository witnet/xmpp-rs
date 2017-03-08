//! A module which exports a few traits for converting types to elements and attributes.

use element::{Element, ElementBuilder};

/// A struct which is used for emitting `Element`s and text nodes into an `Element` without
/// exposing more functionality than necessary.
pub struct ElementEmitter<'a>(&'a mut Element);

impl<'a> ElementEmitter<'a> {
    /// Creates a new `ElementEmitter`.
    pub fn new(root: &'a mut Element) -> ElementEmitter<'a> {
        ElementEmitter(root)
    }

    /// Appends an `Element` to the target.
    pub fn append_child(&mut self, element: Element) {
        self.0.append_child(element);
    }

    /// Appends a text node to the target.
    pub fn append_text_node(&mut self, text: String) {
        self.0.append_text_node(text);
    }
}

/// A trait for types which can be converted to one or multiple `Element`s.
pub trait IntoElements {
    /// Emits this as a sequence of text nodes and `Element`s.
    fn into_elements(self, emitter: &mut ElementEmitter);
}

impl<T: IntoElements> IntoElements for Vec<T> {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        for elem in self {
            elem.into_elements(emitter);
        }
    }
}

impl<'a, T: IntoElements + Clone> IntoElements for &'a [T] {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        self.to_vec().into_elements(emitter);
    }
}

impl<T: IntoElements> IntoElements for Option<T> {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        match self {
            Some(e) => e.into_elements(emitter),
            None => (),
        }
    }
}

impl IntoElements for Element {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self);
    }
}

impl IntoElements for ElementBuilder {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.build());
    }
}

impl IntoElements for String {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_text_node(self);
    }
}

impl<'a> IntoElements for &'a str {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_text_node(self.to_owned());
    }
}

/// A trait for types which can be converted to an attribute value.
pub trait IntoAttributeValue {
    /// Turns this into an attribute string, or None if it shouldn't be added.
    fn into_attribute_value(self) -> Option<String>;
}

impl IntoAttributeValue for String {
    fn into_attribute_value(self) -> Option<String> {
        Some(self.clone())
    }
}

impl<'a> IntoAttributeValue for &'a str {
    fn into_attribute_value(self) -> Option<String> {
        Some((*self).to_owned())
    }
}

impl<T: IntoAttributeValue> IntoAttributeValue for Option<T> {
    fn into_attribute_value(self) -> Option<String> {
        self.and_then(|t| t.into_attribute_value())
    }
}
