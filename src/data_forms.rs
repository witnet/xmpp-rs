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

#[derive(Debug, Clone)]
pub struct Field {
    pub var: String,
    pub type_: String, // TODO: use an enum here.
    pub label: Option<String>,
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
    pub fields: Vec<Field>,
}

impl<'a> TryFrom<&'a Element> for DataForm {
    type Error = Error;

    fn try_from(elem: &'a Element) -> Result<DataForm, Error> {
        if !elem.is("x", ns::DATA_FORMS) {
            return Err(Error::ParseError("This is not a data form element."));
        }

        let type_: DataFormType = match elem.attr("type") {
            Some(type_) => type_.parse()?,
            None => return Err(Error::ParseError("Type attribute on data form is mandatory.")),
        };
        let mut fields = vec!();
        let mut form_type = None;
        for field in elem.children() {
            if field.is("field", ns::DATA_FORMS) {
                let var = field.attr("var").ok_or(Error::ParseError("Field must have a 'var' attribute."))?;
                let field_type = field.attr("type").unwrap_or("text-single");
                let label = field.attr("label").and_then(|label| label.parse().ok());
                let mut values = vec!();
                let mut media = vec!();
                for element in field.children() {
                    if element.is("value", ns::DATA_FORMS) {
                        values.push(element.text());
                    } else if element.is("media", ns::MEDIA_ELEMENT) {
                        match MediaElement::try_from(element) {
                            Ok(media_element) => media.push(media_element),
                            Err(_) => (), // TODO: is it really nice to swallow this error?
                        }
                    } else {
                        return Err(Error::ParseError("Field child isn’t a value or media element."));
                    }
                }
                if var == "FORM_TYPE" && field_type == "hidden" {
                    if form_type != None {
                        return Err(Error::ParseError("More than one FORM_TYPE in a data form."));
                    }
                    if values.len() != 1 {
                        return Err(Error::ParseError("Wrong number of values in FORM_TYPE."));
                    }
                    form_type = Some(values[0].clone());
                }
                fields.push(Field {
                    var: var.to_owned(),
                    type_: field_type.to_owned(),
                    label: label,
                    values: values,
                    media: media,
                });
            } else {
                return Err(Error::ParseError("Unknown field type in data form."));
            }
        }
        Ok(DataForm { type_: type_, form_type: form_type, fields: fields })
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
        assert_eq!(message, "Type attribute on data form is mandatory.");

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
        assert_eq!(message, "Unknown field type in data form.");
    }
}
