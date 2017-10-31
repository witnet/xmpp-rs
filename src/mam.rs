// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::str::FromStr;

use minidom::{Element, IntoAttributeValue};
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

generate_attribute!(DefaultPrefs, "default", {
    Always => "always",
    Never => "never",
    Roster => "roster",
});

#[derive(Debug, Clone)]
pub struct Prefs {
    pub default_: DefaultPrefs,
    pub always: Vec<Jid>,
    pub never: Vec<Jid>,
}

impl TryFrom<Element> for Query {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Query, Error> {
        check_self!(elem, "query", ns::MAM);
        check_no_unknown_attributes!(elem, "query", ["queryid", "node"]);
        let mut form = None;
        let mut set = None;
        for child in elem.children() {
            if child.is("x", ns::DATA_FORMS) {
                form = Some(DataForm::try_from(child.clone())?);
            } else if child.is("set", ns::RSM) {
                set = Some(Set::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in query element."));
            }
        }
        let queryid = get_attr!(elem, "queryid", optional);
        let node = get_attr!(elem, "node", optional);
        Ok(Query { queryid, node, form, set })
    }
}

impl TryFrom<Element> for Result_ {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Result_, Error> {
        check_self!(elem, "result", ns::MAM);
        check_no_unknown_attributes!(elem, "result", ["queryid", "id"]);
        let mut forwarded = None;
        for child in elem.children() {
            if child.is("forwarded", ns::FORWARD) {
                forwarded = Some(Forwarded::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in result element."));
            }
        }
        let forwarded = forwarded.ok_or(Error::ParseError("Mandatory forwarded element missing in result."))?;
        let queryid = get_attr!(elem, "queryid", required);
        let id = get_attr!(elem, "id", required);
        Ok(Result_ {
            queryid,
            id,
            forwarded,
        })
    }
}

impl TryFrom<Element> for Fin {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Fin, Error> {
        check_self!(elem, "fin", ns::MAM);
        check_no_unknown_attributes!(elem, "fin", ["complete"]);
        let mut set = None;
        for child in elem.children() {
            if child.is("set", ns::RSM) {
                set = Some(Set::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in fin element."));
            }
        }
        let set = set.ok_or(Error::ParseError("Mandatory set element missing in fin."))?;
        let complete = match elem.attr("complete") {
            Some(complete) if complete == "true" => true,
            Some(complete) if complete == "false" => false,
            None => false,
            Some(_) => return Err(Error::ParseError("Invalid value for 'complete' attribute.")),
        };
        Ok(Fin { complete, set })
    }
}

impl TryFrom<Element> for Prefs {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Prefs, Error> {
        check_self!(elem, "prefs", ns::MAM);
        check_no_unknown_attributes!(elem, "prefs", ["default"]);
        let mut always = vec!();
        let mut never = vec!();
        for child in elem.children() {
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
        let default_ = get_attr!(elem, "default", required);
        Ok(Prefs { default_, always, never })
    }
}

impl From<Query> for Element {
    fn from(query: Query) -> Element {
        Element::builder("query")
                .ns(ns::MAM)
                .attr("queryid", query.queryid)
                .attr("node", query.node)
                //.append(query.form)
                .append(query.set)
                .build()
    }
}

impl From<Result_> for Element {
    fn from(result: Result_) -> Element {
        Element::builder("result")
                .ns(ns::MAM)
                .attr("queryid", result.queryid)
                .attr("id", result.id)
                .append(result.forwarded)
                .build()
    }
}

impl From<Fin> for Element {
    fn from(fin: Fin) -> Element {
        Element::builder("fin")
                .ns(ns::MAM)
                .attr("complete", if fin.complete { Some("true") } else { None })
                .append(fin.set)
                .build()
    }
}

fn serialise_jid_list(name: &str, jids: Vec<Jid>) -> Option<Element> {
    if jids.is_empty() {
        None
    } else {
        Some(Element::builder(name)
                     .ns(ns::MAM)
                     .append(jids.into_iter()
                                 .map(|jid| Element::builder("jid")
                                                    .ns(ns::MAM)
                                                    .append(jid)
                                                    .build())
                                 .collect::<Vec<_>>())
                     .build())
    }
}

impl From<Prefs> for Element {
    fn from(prefs: Prefs) -> Element {
        Element::builder("prefs")
                .ns(ns::MAM)
                .attr("default", prefs.default_)
                .append(serialise_jid_list("always", prefs.always))
                .append(serialise_jid_list("never", prefs.never))
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_result() {
        #[cfg(not(feature = "component"))]
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
        #[cfg(feature = "component")]
        let elem: Element = r#"
<result xmlns='urn:xmpp:mam:2' queryid='f27' id='28482-98726-73623'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <delay xmlns='urn:xmpp:delay' stamp='2010-07-10T23:08:25Z'/>
    <message xmlns='jabber:component:accept' from="witch@shakespeare.lit" to="macbeth@shakespeare.lit">
      <body>Hail to thee</body>
    </message>
  </forwarded>
</result>
"#.parse().unwrap();
        Result_::try_from(elem).unwrap();
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
        Fin::try_from(elem).unwrap();
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
        Query::try_from(elem).unwrap();
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
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_prefs_get() {
        let elem: Element = "<prefs xmlns='urn:xmpp:mam:2' default='always'/>".parse().unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!());
        assert_eq!(prefs.never, vec!());

        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always/>
  <never/>
</prefs>
"#.parse().unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!());
        assert_eq!(prefs.never, vec!());
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
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!(Jid::from_str("romeo@montague.lit").unwrap()));
        assert_eq!(prefs.never, vec!(Jid::from_str("montague@montague.lit").unwrap()));

        let elem2 = Element::from(prefs.clone());
        println!("{:?}", elem2);
        let prefs2 = Prefs::try_from(elem2).unwrap();
        assert_eq!(prefs.default_, prefs2.default_);
        assert_eq!(prefs.always, prefs2.always);
        assert_eq!(prefs.never, prefs2.never);
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'><coucou/></query>".parse().unwrap();
        let error = Query::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in query element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        let replace = Query { queryid: None, node: None, form: None, set: None };
        let elem2 = replace.into();
        assert_eq!(elem, elem2);
    }
}
