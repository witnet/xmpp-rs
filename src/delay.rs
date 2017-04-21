use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Delay {
    pub from: Option<String>,
    pub stamp: String,
    pub data: Option<String>,
}

pub fn parse_delay(root: &Element) -> Result<Delay, Error> {
    if !root.is("delay", ns::DELAY) {
        return Err(Error::ParseError("This is not a delay element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in delay element."));
    }
    let from = root.attr("from").and_then(|value| value.parse().ok());
    let stamp = root.attr("stamp").ok_or(Error::ParseError("Mandatory argument 'stamp' not present in delay element."))?.to_owned();
    let data = match root.text().as_ref() {
        "" => None,
        text => Some(text.to_owned()),
    };
    Ok(Delay {
        from: from,
        stamp: stamp,
        data: data,
    })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use delay;

    #[test]
    fn test_simple() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>".parse().unwrap();
        let delay = delay::parse_delay(&elem).unwrap();
        assert_eq!(delay.from, Some(String::from("capulet.com")));
        assert_eq!(delay.stamp, "2002-09-10T23:08:25Z");
        assert_eq!(delay.data, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = delay::parse_delay(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a delay element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<delay xmlns='urn:xmpp:delay'><coucou/></delay>".parse().unwrap();
        let error = delay::parse_delay(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in delay element.");
    }
}
