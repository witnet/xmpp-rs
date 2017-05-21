// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::str::FromStr;

use minidom::Element;

use error::Error;
use ns;

use media_element::MediaElement;

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Boolean,
    Fixed,
    Hidden,
    JidMulti,
    JidSingle,
    ListMulti,
    ListSingle,
    TextMulti,
    TextPrivate,
    TextSingle,
}

impl Default for FieldType {
    fn default() -> FieldType {
        FieldType::TextSingle
    }
}

impl FromStr for FieldType {
    type Err = Error;

    fn from_str(s: &str) -> Result<FieldType, Error> {
        Ok(match s {
            "boolean" => FieldType::Boolean,
            "fixed" => FieldType::Fixed,
            "hidden" => FieldType::Hidden,
            "jid-multi" => FieldType::JidMulti,
            "jid-single" => FieldType::JidSingle,
            "list-multi" => FieldType::ListMulti,
            "list-single" => FieldType::ListSingle,
            "text-multi" => FieldType::TextMulti,
            "text-private" => FieldType::TextPrivate,
            "text-single" => FieldType::TextSingle,

            _ => return Err(Error::ParseError("Invalid 'type' attribute in field element.")),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Option_ {
    pub label: Option<String>,
    pub value: String,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum DataFormType {
    Cancel,
    Form,
    Result_,
    Submit,
}

impl FromStr for DataFormType {
    type Err = Error;

    fn from_str(s: &str) -> Result<DataFormType, Error> {
        Ok(match s {
            "cancel" => DataFormType::Cancel,
            "form" => DataFormType::Form,
            "result" => DataFormType::Result_,
            "submit" => DataFormType::Submit,

            _ => return Err(Error::ParseError("Unknown data form type.")),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataForm {
    pub type_: DataFormType,
    pub form_type: Option<String>,
    pub title: Option<String>,
    pub instructions: Option<String>,
    pub fields: Vec<Field>,
}

impl<'a> TryFrom<&'a Element> for DataForm {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<DataForm, Error> {
        if !elem.is("x", ns::DATA_FORMS) {
            return Err(Error::ParseError("This is not a data form element."));
        }
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
                for _ in child.children() {
                    return Err(Error::ParseError("Title element must not have any child."));
                }
                for _ in child.attrs() {
                    return Err(Error::ParseError("Title element must not have any attribute."));
                }
                form.title = Some(child.text());
            } else if child.is("instructions", ns::DATA_FORMS) {
                if form.instructions.is_some() {
                    return Err(Error::ParseError("More than one instructions in form element."));
                }
                for _ in child.children() {
                    return Err(Error::ParseError("instructions element must not have any child."));
                }
                for _ in child.attrs() {
                    return Err(Error::ParseError("instructions element must not have any attribute."));
                }
                form.instructions = Some(child.text());
            } else if child.is("field", ns::DATA_FORMS) {
                let var: String = get_attr!(child, "var", required);
                let field_type = get_attr!(child, "type", default);
                let label = get_attr!(child, "label", optional);

                let is_form_type = var == "FORM_TYPE" && field_type == FieldType::Hidden;
                let is_list = field_type == FieldType::ListSingle || field_type == FieldType::ListMulti;
                let mut field = Field {
                    var: var,
                    type_: field_type,
                    label: label,
                    required: false,
                    options: vec!(),
                    values: vec!(),
                    media: vec!(),
                };
                for element in child.children() {
                    if element.is("value", ns::DATA_FORMS) {
                        for _ in element.children() {
                            return Err(Error::ParseError("Value element must not have any child."));
                        }
                        for _ in element.attrs() {
                            return Err(Error::ParseError("Value element must not have any attribute."));
                        }
                        field.values.push(element.text());
                    } else if element.is("required", ns::DATA_FORMS) {
                        if field.required {
                            return Err(Error::ParseError("More than one required element."));
                        }
                        for _ in element.children() {
                            return Err(Error::ParseError("Required element must not have any child."));
                        }
                        for _ in element.attrs() {
                            return Err(Error::ParseError("Required element must not have any attribute."));
                        }
                        field.required = true;
                    } else if element.is("option", ns::DATA_FORMS) {
                        if !is_list {
                            return Err(Error::ParseError("Option element found in non-list field."));
                        }
                        let label = get_attr!(element, "label", optional);
                        let mut value = None;
                        for child2 in element.children() {
                            if child2.is("value", ns::DATA_FORMS) {
                                if value.is_some() {
                                    return Err(Error::ParseError("More than one value element in option element"));
                                }
                                value = Some(child2.text());
                            } else {
                                return Err(Error::ParseError("Non-value element in option element"));
                            }
                        }
                        let value = value.ok_or(Error::ParseError("No value element in option element"))?;
                        field.options.push(Option_ {
                            label: label,
                            value: value,
                        });
                    } else if element.is("media", ns::MEDIA_ELEMENT) {
                        match MediaElement::try_from(element) {
                            Ok(media_element) => field.media.push(media_element),
                            Err(_) => (), // TODO: is it really nice to swallow this error?
                        }
                    } else {
                        return Err(Error::ParseError("Field child isn’t a value or media element."));
                    }
                }
                if is_form_type {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "<x xmlns='jabber:x:data' type='result'/>".parse().unwrap();
        let form = DataForm::try_from(&elem).unwrap();
        assert_eq!(form.type_, DataFormType::Result_);
        assert!(form.form_type.is_none());
        assert!(form.fields.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<x xmlns='jabber:x:data'/>".parse().unwrap();
        let error = DataForm::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<x xmlns='jabber:x:data' type='coucou'/>".parse().unwrap();
        let error = DataForm::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown data form type.");
    }

    #[test]
    fn test_wrong_child() {
        let elem: Element = "<x xmlns='jabber:x:data' type='cancel'><coucou/></x>".parse().unwrap();
        let error = DataForm::try_from(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in data form element.");
    }
}
