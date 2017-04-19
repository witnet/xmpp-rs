extern crate minidom;

use std::str::FromStr;

use minidom::Element;

use error::Error;
use ns::{DATA_FORMS_NS, MEDIA_ELEMENT_NS};

use media_element::{MediaElement, parse_media_element};

#[derive(Debug)]
pub struct Field {
    pub var: String,
    pub type_: String, // TODO: use an enum here.
    pub label: Option<String>,
    pub values: Vec<String>,
    pub media: Vec<MediaElement>,
}

#[derive(Debug, PartialEq)]
pub enum DataFormType {
    Cancel,
    Form,
    Result_,
    Submit,
}

impl FromStr for DataFormType {
    type Err = Error;

    fn from_str(s: &str) -> Result<DataFormType, Error> {
        if s == "cancel" {
            Ok(DataFormType::Cancel)
        } else if s == "form" {
            Ok(DataFormType::Form)
        } else if s == "result" {
            Ok(DataFormType::Result_)
        } else if s == "submit" {
            Ok(DataFormType::Submit)
        } else {
            Err(Error::ParseError("Unknown data form type."))
        }
    }
}

#[derive(Debug)]
pub struct DataForm {
    pub type_: DataFormType,
    pub form_type: Option<String>,
    pub fields: Vec<Field>,
}

pub fn parse_data_form(root: &Element) -> Result<DataForm, Error> {
    if !root.is("x", DATA_FORMS_NS) {
        return Err(Error::ParseError("This is not a data form element."));
    }

    let type_: DataFormType = match root.attr("type") {
        Some(type_) => type_.parse()?,
        None => return Err(Error::ParseError("Type attribute on data form is mandatory.")),
    };
    let mut fields = vec!();
    let mut form_type = None;
    for field in root.children() {
        if field.is("field", DATA_FORMS_NS) {
            let var = field.attr("var").ok_or(Error::ParseError("Field must have a 'var' attribute."))?;
            let field_type = field.attr("type").unwrap_or("text-single");
            let label = field.attr("label").and_then(|label| label.parse().ok());
            let mut values = vec!();
            let mut media = vec!();
            for element in field.children() {
                if element.is("value", DATA_FORMS_NS) {
                    values.push(element.text());
                } else if element.is("media", MEDIA_ELEMENT_NS) {
                    match parse_media_element(element) {
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

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use data_forms;

    #[test]
    fn test_simple() {
        let elem: Element = "<x xmlns='jabber:x:data' type='result'/>".parse().unwrap();
        let form = data_forms::parse_data_form(&elem).unwrap();
        assert_eq!(form.type_, data_forms::DataFormType::Result_);
        assert!(form.form_type.is_none());
        assert!(form.fields.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<x xmlns='jabber:x:data'/>".parse().unwrap();
        let error = data_forms::parse_data_form(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Type attribute on data form is mandatory.");

        let elem: Element = "<x xmlns='jabber:x:data' type='coucou'/>".parse().unwrap();
        let error = data_forms::parse_data_form(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown data form type.");
    }

    #[test]
    fn test_wrong_child() {
        let elem: Element = "<x xmlns='jabber:x:data' type='cancel'><coucou/></x>".parse().unwrap();
        let error = data_forms::parse_data_form(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown field type in data form.");
    }
}
