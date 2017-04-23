use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Attention;

pub fn parse_attention(root: &Element) -> Result<Attention, Error> {
    if !root.is("attention", ns::ATTENTION) {
        return Err(Error::ParseError("This is not an attention element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in attention element."));
    }
    Ok(Attention)
}

pub fn serialise(_: &Attention) -> Element {
    Element::builder("attention")
            .ns(ns::ATTENTION)
            .build()
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use attention;

    #[test]
    fn test_simple() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'/>".parse().unwrap();
        attention::parse_attention(&elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'><coucou/></attention>".parse().unwrap();
        let error = attention::parse_attention(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in attention element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'/>".parse().unwrap();
        let attention = attention::Attention;
        let elem2 = attention::serialise(&attention);
        assert_eq!(elem, elem2);
    }
}
