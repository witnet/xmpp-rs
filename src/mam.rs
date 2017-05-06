// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use data_forms::DataForm;
use rsm::Set;
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
            form = Some(DataForm::try_from(child)?);
        } else if child.is("set", ns::RSM) {
            set = Some(Set::try_from(child)?);
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
            forwarded = Some(Forwarded::try_from(child)?);
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
            set = Some(Set::try_from(child)?);
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
    //    elem.append_child((&form).into());
    //}
    if let Some(ref set) = query.set {
        elem.append_child(set.into());
    }
    elem
}

pub fn serialise_result(result: &Result_) -> Element {
    let mut elem = Element::builder("result")
                           .ns(ns::MAM)
                           .attr("queryid", result.queryid.clone())
                           .attr("id", result.id.clone())
                           .build();
    elem.append_child((&result.forwarded).into());
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
    elem.append_child((&fin.set).into());
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
    use minidom::Element;
    use error::Error;
    use mam;

    #[test]
    fn test_query() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        mam::parse_query(&elem).unwrap();
    }

    #[test]
    fn test_result() {
        let elem: Element = r#"
<result xmlns='urn:xmpp:mam:2' queryid='f27' id='28482-98726-73623'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <delay xmlns='urn:xmpp:delay' stamp='2010-07-10T23:08:25Z'/>
    <message xmlns='jabber:client' from="witch@shakespeare.lit" to="macbeth@shakespeare.lit">
      <body>Hail to thee</body>
    </message>
  </forwarded>
</result>
"#.parse().unwrap();
        mam::parse_result(&elem).unwrap();
    }

    #[test]
    fn test_fin() {
        let elem: Element = r#"
<fin xmlns='urn:xmpp:mam:2'>
  <set xmlns='http://jabber.org/protocol/rsm'>
    <first index='0'>28482-98726-73623</first>
    <last>09af3-cc343-b409f</last>
  </set>
</fin>
"#.parse().unwrap();
        mam::parse_fin(&elem).unwrap();
    }

    #[test]
    fn test_query_x() {
        let elem: Element = r#"
<query xmlns='urn:xmpp:mam:2'>
  <x xmlns='jabber:x:data' type='submit'>
    <field var='FORM_TYPE' type='hidden'>
      <value>urn:xmpp:mam:2</value>
    </field>
    <field var='with'>
      <value>juliet@capulet.lit</value>
    </field>
  </x>
</query>
"#.parse().unwrap();
        mam::parse_query(&elem).unwrap();
    }

    #[test]
    fn test_query_x_set() {
        let elem: Element = r#"
<query xmlns='urn:xmpp:mam:2'>
  <x xmlns='jabber:x:data' type='submit'>
    <field var='FORM_TYPE' type='hidden'>
      <value>urn:xmpp:mam:2</value>
    </field>
    <field var='start'>
      <value>2010-08-07T00:00:00Z</value>
    </field>
  </x>
  <set xmlns='http://jabber.org/protocol/rsm'>
    <max>10</max>
  </set>
</query>
"#.parse().unwrap();
        mam::parse_query(&elem).unwrap();
    }

    #[test]
    fn test_prefs_get() {
        let elem: Element = "<prefs xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        mam::parse_prefs(&elem).unwrap();

        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always/>
  <never/>
</prefs>
"#.parse().unwrap();
        mam::parse_prefs(&elem).unwrap();
    }

    #[test]
    fn test_prefs_result() {
        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always>
    <jid>romeo@montague.lit</jid>
  </always>
  <never>
    <jid>montague@montague.lit</jid>
  </never>
</prefs>
"#.parse().unwrap();
        mam::parse_prefs(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'><coucou/></query>".parse().unwrap();
        let error = mam::parse_query(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in query element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        let replace = mam::Query { queryid: None, node: None, form: None, set: None };
        let elem2 = mam::serialise_query(&replace);
        assert_eq!(elem, elem2);
    }
}
