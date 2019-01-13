// Copyright (c) 2017 Astro <astro@spaceboyz.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::{Element, Node};

pub trait NamespaceAwareCompare {
    /// Namespace-aware comparison for tests
    fn compare_to(&self, other: &Self) -> bool;
}

impl NamespaceAwareCompare for Node {
    fn compare_to(&self, other: &Self) -> bool {
        match (self, other) {
            (&Node::Element(ref elem1), &Node::Element(ref elem2)) => {
                Element::compare_to(elem1, elem2)
            }
            (&Node::Text(ref text1), &Node::Text(ref text2)) => text1 == text2,
            _ => false,
        }
    }
}

impl NamespaceAwareCompare for Element {
    fn compare_to(&self, other: &Self) -> bool {
        if self.name() == other.name() && self.ns() == other.ns() && self.attrs().eq(other.attrs())
        {
            let child_elems = self.children().count();
            let text_is_whitespace = self
                .texts()
                .all(|text| text.chars().all(char::is_whitespace));
            if child_elems > 0 && text_is_whitespace {
                // Ignore all the whitespace text nodes
                self.children()
                    .zip(other.children())
                    .all(|(node1, node2)| node1.compare_to(node2))
            } else {
                // Compare with text nodes
                self.nodes()
                    .zip(other.nodes())
                    .all(|(node1, node2)| node1.compare_to(node2))
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minidom::Element;

    #[test]
    fn simple() {
        let elem1: Element = "<a a='b'>x <l/> 3</a>".parse().unwrap();
        let elem2: Element = "<a a='b'>x <l/> 3</a>".parse().unwrap();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn wrong_attr_name() {
        let elem1: Element = "<a a='b'>x 3</a>".parse().unwrap();
        let elem2: Element = "<a c='b'>x 3</a>".parse().unwrap();
        assert!(!elem1.compare_to(&elem2));
    }

    #[test]
    fn wrong_attr_value() {
        let elem1: Element = "<a a='b'>x 3</a>".parse().unwrap();
        let elem2: Element = "<a a='c'>x 3</a>".parse().unwrap();
        assert!(!elem1.compare_to(&elem2));
    }

    #[test]
    fn attr_order() {
        let elem1: Element = "<e1 a='b' c='d'/>".parse().unwrap();
        let elem2: Element = "<e1 c='d' a='b'/>".parse().unwrap();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn wrong_texts() {
        let elem1: Element = "<e1>foo</e1>".parse().unwrap();
        let elem2: Element = "<e1>bar</e1>".parse().unwrap();
        assert!(!elem1.compare_to(&elem2));
    }

    #[test]
    fn children() {
        let elem1: Element = "<e1><foo/><bar/></e1>".parse().unwrap();
        let elem2: Element = "<e1><foo/><bar/></e1>".parse().unwrap();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn wrong_children() {
        let elem1: Element = "<e1><foo/></e1>".parse().unwrap();
        let elem2: Element = "<e1><bar/></e1>".parse().unwrap();
        assert!(!elem1.compare_to(&elem2));
    }

    #[test]
    fn xmlns_wrong() {
        let elem1: Element = "<e1 xmlns='ns1'><foo/></e1>".parse().unwrap();
        let elem2: Element = "<e1 xmlns='ns2'><foo/></e1>".parse().unwrap();
        assert!(!elem1.compare_to(&elem2));
    }

    #[test]
    fn xmlns_other_prefix() {
        let elem1: Element = "<e1 xmlns='ns1'><foo/></e1>".parse().unwrap();
        let elem2: Element = "<x:e1 xmlns:x='ns1'><x:foo/></x:e1>".parse().unwrap();
        assert!(elem1.compare_to(&elem2));
    }

    #[test]
    fn xmlns_dup() {
        let elem1: Element = "<e1 xmlns='ns1'><foo/></e1>".parse().unwrap();
        let elem2: Element = "<e1 xmlns='ns1'><foo  xmlns='ns1'/></e1>".parse().unwrap();
        assert!(elem1.compare_to(&elem2));
    }
}
