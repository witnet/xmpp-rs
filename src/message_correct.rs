use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub enum MessageCorrect {
    Replace(String),
}

pub fn parse_message_correct(root: &Element) -> Result<MessageCorrect, Error> {
    if !root.is("replace", ns::MESSAGE_CORRECT) {
        return Err(Error::ParseError("This is not a replace element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in replace element."));
    }
    let id = root.attr("id").unwrap_or("").to_owned();
    Ok(MessageCorrect::Replace(id))
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use message_correct;

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        message_correct::parse_message_correct(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>".parse().unwrap();
        let error = message_correct::parse_message_correct(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }
}
