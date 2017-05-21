// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Set {
    pub after: Option<String>,
    pub before: Option<String>,
    pub count: Option<usize>,
    pub first: Option<String>,
    pub first_index: Option<usize>,
    pub index: Option<usize>,
    pub last: Option<String>,
    pub max: Option<usize>,
}

impl<'a> TryFrom<&'a Element> for Set {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<Set, Error> {
        if !elem.is("set", ns::RSM) {
            return Err(Error::ParseError("This is not a RSM element."));
        }
        let mut set = Set {
            after: None,
            before: None,
            count: None,
            first: None,
            first_index: None,
            index: None,
            last: None,
            max: None,
        };
        for child in elem.children() {
            if child.is("after", ns::RSM) {
                if set.after.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one after."));
                }
                set.after = Some(child.text());
            } else if child.is("before", ns::RSM) {
                if set.before.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one before."));
                }
                set.before = Some(child.text());
            } else if child.is("count", ns::RSM) {
                if set.count.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one count."));
                }
                set.count = Some(child.text().parse()?);
            } else if child.is("first", ns::RSM) {
                if set.first.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one first."));
                }
                set.first_index = match child.attr("index") {
                    Some(index) => Some(index.parse()?),
                    None => None,
                };
                set.first = Some(child.text());
            } else if child.is("index", ns::RSM) {
                if set.index.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one index."));
                }
                set.index = Some(child.text().parse()?);
            } else if child.is("last", ns::RSM) {
                if set.last.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one last."));
                }
                set.last = Some(child.text());
            } else if child.is("max", ns::RSM) {
                if set.max.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one max."));
                }
                set.max = Some(child.text().parse()?);
            } else {
                return Err(Error::ParseError("Unknown child in set element."));
            }
        }
        Ok(set)
    }
}

impl<'a> Into<Element> for &'a Set {
    fn into(self) -> Element {
        let mut elem = Element::builder("set")
                               .ns(ns::RSM)
                               .build();
        if self.after.is_some() {
            elem.append_child(Element::builder("after").ns(ns::RSM).append(self.after.clone()).build());
        }
        if self.before.is_some() {
            elem.append_child(Element::builder("before").ns(ns::RSM).append(self.before.clone()).build());
        }
        if self.count.is_some() {
            elem.append_child(Element::builder("count").ns(ns::RSM).append(format!("{}", self.count.unwrap())).build());
        }
        if self.first.is_some() {
            elem.append_child(Element::builder("first")
                                      .ns(ns::RSM)
                                      .attr("index", self.first_index.map(|index| format!("{}", index)))
                                      .append(self.first.clone()).build());
        }
        if self.index.is_some() {
            elem.append_child(Element::builder("index").ns(ns::RSM).append(format!("{}", self.index.unwrap())).build());
        }
        if self.last.is_some() {
            elem.append_child(Element::builder("last").ns(ns::RSM).append(self.last.clone()).build());
        }
        if self.max.is_some() {
            elem.append_child(Element::builder("max").ns(ns::RSM).append(format!("{}", self.max.unwrap())).build());
        }
        elem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>".parse().unwrap();
        let set = Set::try_from(&elem).unwrap();
        assert_eq!(set.after, None);
        assert_eq!(set.before, None);
        assert_eq!(set.count, None);
        match set.first {
            Some(_) => panic!(),
            None => (),
        }
        assert_eq!(set.index, None);
        assert_eq!(set.last, None);
        assert_eq!(set.max, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = Set::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a RSM element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'><coucou/></set>".parse().unwrap();
        let error = Set::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in set element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>".parse().unwrap();
        let rsm = Set {
            after: None,
            before: None,
            count: None,
            first: None,
            first_index: None,
            index: None,
            last: None,
            max: None,
        };
        let elem2 = (&rsm).into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_first_index() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'><first index='4'>coucou</first></set>".parse().unwrap();
        let set = Set::try_from(&elem).unwrap();
        assert_eq!(set.first, Some(String::from("coucou")));
        assert_eq!(set.first_index, Some(4));

        let set2 = Set {
            after: None,
            before: None,
            count: None,
            first: Some(String::from("coucou")),
            first_index: Some(4),
            index: None,
            last: None,
            max: None,
        };
        let elem2 = (&set2).into();
        assert_eq!(elem, elem2);
    }
}
