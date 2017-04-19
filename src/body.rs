use minidom::Element;

use error::Error;

// TODO: also support components and servers.
use ns::JABBER_CLIENT_NS;

#[derive(Debug)]
pub struct Body {
    body: String,
}

pub fn parse_body(root: &Element) -> Result<Body, Error> {
    if !root.is("body", JABBER_CLIENT_NS) {
        return Err(Error::ParseError("This is not a body element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in body element."));
    }
    Ok(Body { body: root.text() })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use chatstates;

    #[test]
    fn test_simple() {
        let elem: Element = "<active xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        chatstates::parse_chatstate(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<coucou xmlns='http://jabber.org/protocol/chatstates'/>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown chatstate element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<gone xmlns='http://jabber.org/protocol/chatstates'><coucou/></gone>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in chatstate element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<inactive xmlns='http://jabber.org/protocol/chatstates' coucou=''/>".parse().unwrap();
        let error = chatstates::parse_chatstate(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in chatstate element.");
    }
}
