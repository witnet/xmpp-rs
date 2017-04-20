use minidom::Element;

use error::Error;

use ns;

#[derive(Debug)]
pub struct Body {
    pub body: String,
}

pub fn parse_body(root: &Element) -> Result<Body, Error> {
    // TODO: also support components and servers.
    if !root.is("body", ns::JABBER_CLIENT) {
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
    use body;

    #[test]
    fn test_simple() {
        let elem: Element = "<body xmlns='jabber:client'/>".parse().unwrap();
        body::parse_body(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<body xmlns='jabber:server'/>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a body element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<body xmlns='jabber:client'><coucou/></body>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in body element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<body xmlns='jabber:client' coucou=''/>".parse().unwrap();
        let error = body::parse_body(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in body element.");
    }
}
