// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(missing_docs)]

use try_from::TryFrom;

use minidom::Element;

use error::Error;
use ns;

use media_element::MediaElement;

generate_element!(
    Option_, "option", DATA_FORMS,
    attributes: [
        label: Option<String> = "label" => optional
    ],
    children: [
        value: Required<String> = ("value", DATA_FORMS) => String
    ]
);

generate_attribute!(FieldType, "type", {
    Boolean => "boolean",
    Fixed => "fixed",
    Hidden => "hidden",
    JidMulti => "jid-multi",
    JidSingle => "jid-single",
    ListMulti => "list-multi",
    ListSingle => "list-single",
    TextMulti => "text-multi",
    TextPrivate => "text-private",
    TextSingle => "text-single",
}, Default = TextSingle);

#[derive(Debug, Clone)]
pub struct Field {
    pub var: String,
    pub type_: FieldType,
    pub label: Option<String>,
    pub required: bool,
    pub options: Vec<Option_>,
    pub values: Vec<String>,
    pub media: Vec<MediaElement>,
}

impl Field {
    fn is_list(&self) -> bool {
        self.type_ == FieldType::ListSingle ||
        self.type_ == FieldType::ListMulti
    }
}

impl TryFrom<Element> for Field {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Field, Error> {
        check_self!(elem, "field", DATA_FORMS);
        check_no_unknown_attributes!(elem, "field", ["label", "type", "var"]);
        let mut field = Field {
            var: get_attr!(elem, "var", required),
            type_: get_attr!(elem, "type", default),
            label: get_attr!(elem, "label", optional),
            required: false,
            options: vec!(),
            values: vec!(),
            media: vec!(),
        };
        for element in elem.children() {
            if element.is("value", ns::DATA_FORMS) {
                check_no_children!(element, "value");
                check_no_attributes!(element, "value");
                field.values.push(element.text());
            } else if element.is("required", ns::DATA_FORMS) {
                if field.required {
                    return Err(Error::ParseError("More than one required element."));
                }
                check_no_children!(element, "required");
                check_no_attributes!(element, "required");
                field.required = true;
            } else if element.is("option", ns::DATA_FORMS) {
                if !field.is_list() {
                    return Err(Error::ParseError("Option element found in non-list field."));
                }
                let option = Option_::try_from(element.clone())?;
                field.options.push(option);
            } else if element.is("media", ns::MEDIA_ELEMENT) {
                let media_element = MediaElement::try_from(element.clone())?;
                field.media.push(media_element);
            } else {
                return Err(Error::ParseError("Field child isn’t a value, option or media element."));
            }
        }
        Ok(field)
    }
}

impl From<Field> for Element {
    fn from(field: Field) -> Element {
        Element::builder("field")
                .ns(ns::DATA_FORMS)
                .attr("var", field.var)
                .attr("type", field.type_)
                .attr("label", field.label)
                .append(if field.required { Some(Element::builder("required").ns(ns::DATA_FORMS).build()) } else { None })
                .append(field.options)
                .append(field.values.into_iter().map(|value| {
                     Element::builder("value").ns(ns::DATA_FORMS).append(value).build()
                 }).collect::<Vec<_>>())
                .append(field.media)
                .build()
    }
}

generate_attribute!(DataFormType, "type", {
    Cancel => "cancel",
    Form => "form",
    Result_ => "result",
    Submit => "submit",
});

#[derive(Debug, Clone)]
pub struct DataForm {
    pub type_: DataFormType,
    pub form_type: Option<String>,
    pub title: Option<String>,
    pub instructions: Option<String>,
    pub fields: Vec<Field>,
}

impl TryFrom<Element> for DataForm {
    type Err = Error;

    fn try_from(elem: Element) -> Result<DataForm, Error> {
        check_self!(elem, "x", DATA_FORMS);
        check_no_unknown_attributes!(elem, "x", ["type"]);
        let type_ = get_attr!(elem, "type", required);
        let mut form = DataForm {
            type_: type_,
            form_type: None,
            title: None,
            instructions: None,
            fields: vec!(),
        };
        for child in elem.children() {
            if child.is("title", ns::DATA_FORMS) {
                if form.title.is_some() {
                    return Err(Error::ParseError("More than one title in form element."));
                }
                check_no_children!(child, "title");
                check_no_attributes!(child, "title");
                form.title = Some(child.text());
            } else if child.is("instructions", ns::DATA_FORMS) {
                if form.instructions.is_some() {
                    return Err(Error::ParseError("More than one instructions in form element."));
                }
                check_no_children!(child, "instructions");
                check_no_attributes!(child, "instructions");
                form.instructions = Some(child.text());
            } else if child.is("field", ns::DATA_FORMS) {
                let field = Field::try_from(child.clone())?;
                if field.var == "FORM_TYPE" && field.type_ == FieldType::Hidden {
                    if form.form_type.is_some() {
                        return Err(Error::ParseError("More than one FORM_TYPE in a data form."));
                    }
                    if field.values.len() != 1 {
                        return Err(Error::ParseError("Wrong number of values in FORM_TYPE."));
                    }
                    form.form_type = Some(field.values[0].clone());
                }
                form.fields.push(field);
            } else {
                return Err(Error::ParseError("Unknown child in data form element."));
            }
        }
        Ok(form)
    }
}

impl From<DataForm> for Element {
    fn from(form: DataForm) -> Element {
        Element::builder("x")
                .ns(ns::DATA_FORMS)
                .attr("type", form.type_)
                .append(form.title.map(|title| Element::builder("title").ns(ns::DATA_FORMS).append(title)))
                .append(form.instructions.map(|text| Element::builder("instructions").ns(ns::DATA_FORMS).append(text)))
                .append(form.fields)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<x xmlns='jabber:x:data' type='result'/>".parse().unwrap();
        let form = DataForm::try_from(elem).unwrap();
        assert_eq!(form.type_, DataFormType::Result_);
        assert!(form.form_type.is_none());
        assert!(form.fields.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<x xmlns='jabber:x:data'/>".parse().unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<x xmlns='jabber:x:data' type='coucou'/>".parse().unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'type' attribute.");
    }

    #[test]
    fn test_wrong_child() {
        let elem: Element = "<x xmlns='jabber:x:data' type='cancel'><coucou/></x>".parse().unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in data form element.");
    }

    #[test]
    fn option() {
        let elem: Element = "<option xmlns='jabber:x:data' label='Coucou !'><value>coucou</value></option>".parse().unwrap();
        let option = Option_::try_from(elem).unwrap();
        assert_eq!(&option.label.unwrap(), "Coucou !");
        assert_eq!(&option.value, "coucou");

        let elem: Element = "<option xmlns='jabber:x:data' label='Coucou !'/>".parse().unwrap();
        let error = Option_::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Missing child value in option element.");

        let elem: Element = "<option xmlns='jabber:x:data' label='Coucou !'><value>coucou</value><value>error</value></option>".parse().unwrap();
        let error = Option_::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Element option must not have more than one value child.");
    }
}
