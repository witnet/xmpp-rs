use std::str::FromStr;

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub enum Stanza {
    Iq,
    Message,
}

impl Default for Stanza {
    fn default() -> Stanza {
        Stanza::Iq
    }
}

impl FromStr for Stanza {
    type Err = Error;

    fn from_str(s: &str) -> Result<Stanza, Error> {
        if s == "iq" {
            Ok(Stanza::Iq)
        } else if s == "message" {
            Ok(Stanza::Message)
        } else {
            Err(Error::ParseError("Unknown 'stanza' attribute."))
        }
    }
}

#[derive(Debug, Clone)]
pub enum IBB {
    Open { block_size: u16, sid: String, stanza: Stanza },
    Data(u16, String, Vec<u8>),
    Close(String),
}

fn optional_attr<T: FromStr>(root: &Element, attr: &str) -> Option<T> {
    root.attr(attr)
        .and_then(|value| value.parse().ok())
}

fn required_attr<T: FromStr>(root: &Element, attr: &str, err: Error) -> Result<T, Error> {
    optional_attr(root, attr).ok_or(err)
}

pub fn parse_ibb(root: &Element) -> Result<IBB, Error> {
    if root.is("open", ns::IBB) {
        let block_size = required_attr(root, "block-size", Error::ParseError("Required attribute 'block-size' missing in open element."))?;
        let sid = required_attr(root, "sid", Error::ParseError("Required attribute 'sid' missing in open element."))?;
        let stanza = root.attr("stanza")
                         .and_then(|value| value.parse().ok())
                         .unwrap_or(Default::default());
        for _ in root.children() {
            return Err(Error::ParseError("Unknown child in open element."));
        }
        Ok(IBB::Open {
            block_size: block_size,
            sid: sid,
            stanza: stanza
        })
    } else {
        Err(Error::ParseError("This is not an ibb element."))
    }
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use ibb;

    #[test]
    fn test_simple() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou'/>".parse().unwrap();
        ibb::parse_ibb(&elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb'/>".parse().unwrap();
        let error = ibb::parse_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing in open element.");

        // TODO: maybe make a better error message here.
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='-5'/>".parse().unwrap();
        let error = ibb::parse_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing in open element.");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128'/>".parse().unwrap();
        let error = ibb::parse_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing in open element.");
    }

    #[test]
    #[ignore]
    fn test_invalid_stanza() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = ibb::parse_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Wrong value for 'stanza' attribute in open.");
    }
}
