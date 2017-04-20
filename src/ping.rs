use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Ping {
}

pub fn parse_ping(root: &Element) -> Result<Ping, Error> {
    if !root.is("ping", ns::PING) {
        return Err(Error::ParseError("This is not a ping element."));
    }

    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in ping element."));
    }
    Ok(Ping {  })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use ping;

    #[test]
    fn test_simple() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'/>".parse().unwrap();
        ping::parse_ping(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping'><coucou/></ping>".parse().unwrap();
        let error = ping::parse_ping(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in ping element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_attribute() {
        let elem: Element = "<ping xmlns='urn:xmpp:ping' coucou=''/>".parse().unwrap();
        let error = ping::parse_ping(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in ping element.");
    }
}
