// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct First {
    pub index: Option<usize>,
    pub base: String,
}

#[derive(Debug, Clone)]
pub struct Set {
    pub after: Option<String>,
    pub before: Option<String>,
    pub count: Option<usize>,
    pub first: Option<First>,
    pub index: Option<usize>,
    pub last: Option<String>,
    pub max: Option<usize>,
}

pub fn parse_rsm(root: &Element) -> Result<Set, Error> {
    if !root.is("set", ns::RSM) {
        return Err(Error::ParseError("This is not a RSM element."));
    }
    let mut after = None;
    let mut before = None;
    let mut count = None;
    let mut first = None;
    let mut index = None;
    let mut last = None;
    let mut max = None;
    for child in root.children() {
        if child.is("after", ns::RSM) {
            if after.is_some() {
                return Err(Error::ParseError("Set can’t have more than one after."));
            }
            after = Some(child.text());
        } else if child.is("before", ns::RSM) {
            if before.is_some() {
                return Err(Error::ParseError("Set can’t have more than one before."));
            }
            before = Some(child.text());
        } else if child.is("count", ns::RSM) {
            if count.is_some() {
                return Err(Error::ParseError("Set can’t have more than one count."));
            }
            count = Some(child.text().parse()?);
        } else if child.is("first", ns::RSM) {
            if first.is_some() {
                return Err(Error::ParseError("Set can’t have more than one first."));
            }
            first = Some(First {
                index: match child.attr("index") {
                    Some(index) => Some(index.parse()?),
                    None => None,
                },
                base: child.text(),
            });
        } else if child.is("index", ns::RSM) {
            if index.is_some() {
                return Err(Error::ParseError("Set can’t have more than one index."));
            }
            index = Some(child.text().parse()?);
        } else if child.is("last", ns::RSM) {
            if last.is_some() {
                return Err(Error::ParseError("Set can’t have more than one last."));
            }
            last = Some(child.text());
        } else if child.is("max", ns::RSM) {
            if max.is_some() {
                return Err(Error::ParseError("Set can’t have more than one max."));
            }
            max = Some(child.text().parse()?);
        } else {
            return Err(Error::ParseError("Unknown child in set element."));
        }
    }
    Ok(Set {
        after: after,
        before: before,
        count: count,
        first: first,
        index: index,
        last: last,
        max: max,
    })
}

pub fn serialise(rsm: &Set) -> Element {
    let mut elem = Element::builder("set")
                           .ns(ns::RSM)
                           .build();
    if rsm.after.is_some() {
        elem.append_child(Element::builder("after").ns(ns::RSM).append(rsm.after.clone()).build());
    }
    if rsm.before.is_some() {
        elem.append_child(Element::builder("before").ns(ns::RSM).append(rsm.before.clone()).build());
    }
    if rsm.count.is_some() {
        elem.append_child(Element::builder("count").ns(ns::RSM).append(format!("{}", rsm.count.unwrap())).build());
    }
    if rsm.first.is_some() {
        let first = rsm.first.clone().unwrap();
        elem.append_child(Element::builder("first")
                                  .ns(ns::RSM)
                                  .attr("index", match first.index {
                                       Some(index) => Some(format!("{}", index)),
                                       None => None,
                                   })
                                  .append(first.base.clone()).build());
    }
    if rsm.index.is_some() {
        elem.append_child(Element::builder("index").ns(ns::RSM).append(format!("{}", rsm.index.unwrap())).build());
    }
    if rsm.last.is_some() {
        elem.append_child(Element::builder("last").ns(ns::RSM).append(rsm.last.clone()).build());
    }
    if rsm.max.is_some() {
        elem.append_child(Element::builder("max").ns(ns::RSM).append(format!("{}", rsm.max.unwrap())).build());
    }
    elem
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use rsm;

    #[test]
    fn test_simple() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>".parse().unwrap();
        let set = rsm::parse_rsm(&elem).unwrap();
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
        let error = rsm::parse_rsm(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a RSM element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'><coucou/></set>".parse().unwrap();
        let error = rsm::parse_rsm(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in set element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>".parse().unwrap();
        let rsm = rsm::Set {
            after: None,
            before: None,
            count: None,
            first: None,
            index: None,
            last: None,
            max: None,
        };
        let elem2 = rsm::serialise(&rsm);
        assert_eq!(elem, elem2);
    }
}
