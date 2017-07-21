// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use try_from::TryFrom;

use minidom::Element;

use error::Error;

use data_forms::DataForm;

use ns;

#[derive(Debug, Clone)]
pub struct Query {
    pub fields: HashMap<String, String>,
    pub registered: bool,
    pub remove: bool,
    pub form: Option<DataForm>,
    // Not yet implemented.
    //pub oob: Option<Oob>,
}

impl TryFrom<Element> for Query {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Query, Error> {
        if !elem.is("query", ns::REGISTER) {
            return Err(Error::ParseError("This is not an ibr element."));
        }
        let mut query = Query {
            registered: false,
            fields: HashMap::new(),
            remove: false,
            form: None,
        };
        for child in elem.children() {
            let namespace = child.ns().unwrap();
            if namespace == ns::REGISTER {
                let name = child.name();
                let fields = vec!["address", "city", "date", "email", "first", "instructions",
                                  "key", "last", "misc", "name", "nick", "password", "phone",
                                  "state", "text", "url", "username", "zip"];
                if fields.binary_search(&name).is_ok() {
                    query.fields.insert(name.to_owned(), child.text());
                } else if name == "registered" {
                    query.registered = true;
                } else if name == "remove" {
                    query.remove = true;
                } else {
                    return Err(Error::ParseError("Wrong field in ibr element."));
                }
            } else if child.is("x", ns::DATA_FORMS) {
                query.form = Some(DataForm::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in ibr element."));
            }
        }
        Ok(query)
    }
}

impl From<Query> for Element {
    fn from(query: Query) -> Element {
        Element::builder("query")
                .ns(ns::REGISTER)
                .append(if query.registered { Some(Element::builder("registered").ns(ns::REGISTER)) } else { None })
                .append(query.fields.into_iter().map(|(name, value)| {
                     Element::builder(name).ns(ns::REGISTER).append(value)
                 }).collect::<Vec<_>>())
                .append(if query.remove { Some(Element::builder("remove").ns(ns::REGISTER)) } else { None })
                .append(query.form)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='jabber:iq:register'/>".parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_ex2() {
        let elem: Element = r#"
<query xmlns='jabber:iq:register'>
  <instructions>
    Choose a username and password for use with this service.
    Please also provide your email address.
  </instructions>
  <username/>
  <password/>
  <email/>
</query>
"#.parse().unwrap();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        assert_eq!(query.fields["instructions"], "\n    Choose a username and password for use with this service.\n    Please also provide your email address.\n  ");
        assert_eq!(query.fields["username"], "");
        assert_eq!(query.fields["password"], "");
        assert_eq!(query.fields["email"], "");
        assert_eq!(query.fields.contains_key("name"), false);

        // FIXME: HashMap doesn’t keep the order right.
        //let elem2 = query.into();
        //assert_eq!(elem, elem2);
    }

    #[test]
    fn test_ex9() {
        let elem: Element = r#"
<query xmlns='jabber:iq:register'>
  <instructions>
    Use the enclosed form to register. If your Jabber client does not
    support Data Forms, visit http://www.shakespeare.lit/contests.php
  </instructions>
  <x xmlns='jabber:x:data' type='form'>
    <title>Contest Registration</title>
    <instructions>
      Please provide the following information
      to sign up for our special contests!
    </instructions>
    <field type='hidden' var='FORM_TYPE'>
      <value>jabber:iq:register</value>
    </field>
    <field label='Given Name' var='first'>
      <required/>
    </field>
    <field label='Family Name' var='last'>
      <required/>
    </field>
    <field label='Email Address' var='email'>
      <required/>
    </field>
    <field type='list-single' label='Gender' var='x-gender'>
      <option label='Male'><value>M</value></option>
      <option label='Female'><value>F</value></option>
    </field>
  </x>
</query>
"#.parse().unwrap();
        let elem1 = elem.clone();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        assert!(!query.fields["instructions"].is_empty());
        let form = query.form.clone().unwrap();
        assert!(!form.instructions.unwrap().is_empty());
        assert!(form.fields.binary_search_by(|field| field.var.cmp(&String::from("first"))).is_ok());
        assert!(form.fields.binary_search_by(|field| field.var.cmp(&String::from("x-gender"))).is_ok());
        assert!(form.fields.binary_search_by(|field| field.var.cmp(&String::from("coucou"))).is_err());
        let elem2 = query.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_ex10() {
        let elem: Element = r#"
<query xmlns='jabber:iq:register'>
  <x xmlns='jabber:x:data' type='submit'>
    <field type='hidden' var='FORM_TYPE'>
      <value>jabber:iq:register</value>
    </field>
    <field label='Given Name' var='first'>
      <value>Juliet</value>
    </field>
    <field label='Family Name' var='last'>
      <value>Capulet</value>
    </field>
    <field label='Email Address' var='email'>
      <value>juliet@capulet.com</value>
    </field>
    <field type='list-single' label='Gender' var='x-gender'>
      <value>F</value>
    </field>
  </x>
</query>
"#.parse().unwrap();
        let elem1 = elem.clone();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        for _ in &query.fields {
            panic!();
        }
        let elem2 = query.into();
        assert_eq!(elem1, elem2);
    }
}
