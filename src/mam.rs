use minidom::Element;
use jid::Jid;

use error::Error;

use data_forms;
use data_forms::DataForm;
use rsm;
use rsm::Set;
use forwarding;
use forwarding::Forwarded;

use ns;

#[derive(Debug, Clone)]
pub struct Query {
    pub queryid: Option<String>,
    pub node: Option<String>,
    pub form: Option<DataForm>,
    pub set: Option<Set>,
}

#[derive(Debug, Clone)]
pub struct Result_ {
    pub queryid: String,
    pub id: String,
    pub forwarded: Forwarded,
}

#[derive(Debug, Clone)]
pub struct Fin {
    pub complete: bool,
    pub set: Set,
}

#[derive(Debug, Clone)]
pub enum DefaultPrefs {
    Always,
    Never,
    Roster,
}

#[derive(Debug, Clone)]
pub struct Prefs {
    pub default_: Option<DefaultPrefs>,
    pub always: Vec<Jid>,
    pub never: Vec<Jid>,
}

pub fn parse_query(root: &Element) -> Result<Query, Error> {
    if !root.is("query", ns::MAM) {
        return Err(Error::ParseError("This is not a query element."));
    }
    let mut form = None;
    let mut set = None;
    for child in root.children() {
        if child.is("x", ns::DATA_FORMS) {
            form = Some(data_forms::parse_data_form(child)?);
        } else if child.is("set", ns::RSM) {
            set = Some(rsm::parse_rsm(child)?);
        } else {
            return Err(Error::ParseError("Unknown child in query element."));
        }
    }
    let queryid = match root.attr("queryid") {
        Some(queryid) => Some(queryid.to_owned()),
        None => None,
    };
    let node = match root.attr("node") {
        Some(node) => Some(node.to_owned()),
        None => None,
    };
    Ok(Query { queryid, node, form, set })
}

pub fn parse_result(root: &Element) -> Result<Result_, Error> {
    if !root.is("result", ns::MAM) {
        return Err(Error::ParseError("This is not a result element."));
    }
    let mut forwarded = None;
    for child in root.children() {
        if child.is("forwarded", ns::FORWARD) {
            forwarded = Some(forwarding::parse_forwarded(child)?);
        } else {
            return Err(Error::ParseError("Unknown child in result element."));
        }
    }
    let queryid = match root.attr("queryid") {
        Some(queryid) => queryid.to_owned(),
        None => return Err(Error::ParseError("No 'queryid' attribute present in result.")),
    };
    let id = match root.attr("id") {
        Some(id) => id.to_owned(),
        None => return Err(Error::ParseError("No 'id' attribute present in result.")),
    };
    if forwarded.is_none() {
        return Err(Error::ParseError("Mandatory forwarded element missing in result."));
    }
    let forwarded = forwarded.unwrap();
    Ok(Result_ {
        queryid,
        id,
        forwarded,
    })
}

pub fn parse_fin(root: &Element) -> Result<Fin, Error> {
    if !root.is("fin", ns::MAM) {
        return Err(Error::ParseError("This is not a fin element."));
    }
    let mut set = None;
    for child in root.children() {
        if child.is("set", ns::RSM) {
            set = Some(rsm::parse_rsm(child)?);
        } else {
            return Err(Error::ParseError("Unknown child in fin element."));
        }
    }
    let complete = match root.attr("complete") {
        Some(complete) => complete == "true",
        None => false,
    };
    if set.is_none() {
        return Err(Error::ParseError("Mandatory set element missing in fin."));
    }
    let set = set.unwrap();
    Ok(Fin { complete, set })
}

pub fn parse_prefs(root: &Element) -> Result<Prefs, Error> {
    if !root.is("prefs", ns::MAM) {
        return Err(Error::ParseError("This is not a prefs element."));
    }
    let mut always = vec!();
    let mut never = vec!();
    for child in root.children() {
        if child.is("always", ns::MAM) {
            for jid_elem in child.children() {
                if !jid_elem.is("jid", ns::MAM) {
                    return Err(Error::ParseError("Invalid jid element in always."));
                }
                always.push(jid_elem.text().parse()?);
            }
        } else if child.is("never", ns::MAM) {
            for jid_elem in child.children() {
                if !jid_elem.is("jid", ns::MAM) {
                    return Err(Error::ParseError("Invalid jid element in never."));
                }
                never.push(jid_elem.text().parse()?);
            }
        } else {
            return Err(Error::ParseError("Unknown child in prefs element."));
        }
    }
    let default_ = match root.attr("default") {
        Some("always") => Some(DefaultPrefs::Always),
        Some("never") => Some(DefaultPrefs::Never),
        Some("roster") => Some(DefaultPrefs::Roster),
        None => None,

        _ => return Err(Error::ParseError("Invalid 'default' attribute present in prefs.")),
    };
    Ok(Prefs { default_, always, never })
}

pub fn serialise_query(query: &Query) -> Element {
    let mut elem = Element::builder("query")
                           .ns(ns::MAM)
                           .attr("queryid", query.queryid.clone())
                           .attr("node", query.node.clone())
                           .build();
    //if let Some(form) = query.form {
    //    elem.append_child(data_forms::serialise(&form));
    //}
    if let Some(ref set) = query.set {
        elem.append_child(rsm::serialise(&set));
    }
    elem
}

pub fn serialise_result(result: &Result_) -> Element {
    let mut elem = Element::builder("result")
                           .ns(ns::MAM)
                           .attr("queryid", result.queryid.clone())
                           .attr("id", result.id.clone())
                           .build();
    elem.append_child(forwarding::serialise(&result.forwarded));
    elem
}

pub fn serialise_fin(fin: &Fin) -> Element {
    let mut elem = Element::builder("fin")
                           .ns(ns::MAM)
                           .attr("complete", match fin.complete {
                                true => Some("true"),
                                false => None,
                            })
                           .build();
    elem.append_child(rsm::serialise(&fin.set));
    elem
}

pub fn serialise_prefs(prefs: &Prefs) -> Element {
    let mut elem = Element::builder("prefs")
                           .ns(ns::MAM)
                           .attr("default", match prefs.default_ {
                                Some(DefaultPrefs::Always) => Some("always"),
                                Some(DefaultPrefs::Never) => Some("never"),
                                Some(DefaultPrefs::Roster) => Some("roster"),
                                None => None,
                            })
                           .build();
    if !prefs.always.is_empty() {
        let mut always = Element::builder("always")
                                 .ns(ns::RSM)
                                 .build();
        for jid in prefs.always.clone() {
            always.append_child(Element::builder("jid")
                                        .ns(ns::RSM)
                                        .append(String::from(jid))
                                        .build());
        }
        elem.append_child(always);
    }
    if !prefs.never.is_empty() {
        let mut never = Element::builder("never")
                                 .ns(ns::RSM)
                                 .build();
        for jid in prefs.never.clone() {
            never.append_child(Element::builder("jid")
                                        .ns(ns::RSM)
                                        .append(String::from(jid))
                                        .build());
        }
        elem.append_child(never);
    }
    elem
}

#[cfg(test)]
mod tests {
    /*
    use minidom::Element;
    use error::Error;
    use mam;

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        mam::parse_query(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>".parse().unwrap();
        let error = mam::parse_query(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = mam::parse_query(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No 'id' attribute present in replace.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        let replace = mam::Query { id: String::from("coucou") };
        let elem2 = mam::serialise(&replace);
        assert_eq!(elem, elem2);
    }
    */
}
