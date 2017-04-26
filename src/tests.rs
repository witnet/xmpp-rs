use std::io::Cursor;

use std::iter::Iterator;

use xml::reader::EventReader;
use xml::writer::EventWriter;

use element::Element;

const TEST_STRING: &'static str = r#"<?xml version="1.0" encoding="utf-8"?><root xmlns="root_ns" a="b" xml:lang="en">meow<child c="d" /><child xmlns="child_ns" d="e" xml:lang="fr" />nya</root>"#;

fn build_test_tree() -> Element {
    let mut root = Element::builder("root")
                           .ns("root_ns")
                           .attr("xml:lang", "en")
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
                              .attr("xml:lang", "fr")
                              .build();
    root.append_child(other_child);
    root.append_text_node("nya");
    root
}

#[test]
fn reader_works() {
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
                       .append(Element::builder("child"))
                       .append("e")
                       .build();
    assert_eq!(elem.name(), "a");
    assert_eq!(elem.ns(), Some("b"));
    assert_eq!(elem.attr("c"), Some("d"));
    assert_eq!(elem.attr("x"), None);
    assert_eq!(elem.text(), "e");
    assert!(elem.has_child("child", "b"));
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

#[test]
fn namespace_propagation_works() {
    let mut root = Element::builder("root").ns("root_ns").build();
    let mut child = Element::bare("child");
    let grandchild = Element::bare("grandchild");
    child.append_child(grandchild);
    root.append_child(child);
    assert_eq!(root.get_child("child", "root_ns").unwrap().ns(), root.ns());
    assert_eq!(root.get_child("child", "root_ns").unwrap()
                   .get_child("grandchild", "root_ns").unwrap()
                   .ns(), root.ns());
}

#[test]
fn two_elements_with_same_arguments_different_order_are_equal() {
    let elem1: Element = "<a b='a' c=''/>".parse().unwrap();
    let elem2: Element = "<a c='' b='a'/>".parse().unwrap();
    assert_eq!(elem1, elem2);

    let elem1: Element = "<a b='a' c=''/>".parse().unwrap();
    let elem2: Element = "<a c='d' b='a'/>".parse().unwrap();
    assert_ne!(elem1, elem2);
}

#[test]
fn namespace_attributes_works() {
    let mut reader = EventReader::new(Cursor::new(TEST_STRING));
    let root = Element::from_reader(&mut reader).unwrap();
    assert_eq!("en", root.attr("xml:lang").unwrap());
    assert_eq!("fr", root.get_child("child", "child_ns").unwrap().attr("xml:lang").unwrap());
}
