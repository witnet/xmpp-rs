use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Replace {
    pub id: String,
}

pub fn parse_replace(root: &Element) -> Result<Replace, Error> {
    if !root.is("replace", ns::MESSAGE_CORRECT) {
        return Err(Error::ParseError("This is not a replace element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in replace element."));
    }
    let id = match root.attr("id") {
        Some(id) => id.to_owned(),
        None => return Err(Error::ParseError("No 'id' attribute present in replace.")),
    };
    Ok(Replace { id: id })
}

pub fn serialise(replace: &Replace) -> Element {
    Element::builder("replace")
            .ns(ns::MESSAGE_CORRECT)
            .attr("id", replace.id.clone())
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use message_correct;

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        message_correct::parse_replace(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>".parse().unwrap();
        let error = message_correct::parse_replace(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = message_correct::parse_replace(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No 'id' attribute present in replace.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>".parse().unwrap();
        let replace = message_correct::Replace { id: String::from("coucou") };
        let elem2 = message_correct::serialise(&replace);
        assert_eq!(elem, elem2);
    }
}
