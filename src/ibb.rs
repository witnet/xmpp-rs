use std::str::FromStr;

use minidom::Element;
use base64;

use error::Error;

use ns;

#[derive(Debug, Clone, PartialEq)]
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
        Ok(match s {
            "iq" => Stanza::Iq,
            "message" => Stanza::Message,

            _ => return Err(Error::ParseError("Invalid 'stanza' attribute.")),
        })
    }
}

#[derive(Debug, Clone)]
pub enum IBB {
    Open {
        block_size: u16,
        sid: String,
        stanza: Stanza,
    },
    Data {
        seq: u16,
        sid: String,
        data: Vec<u8>,
    },
    Close {
        sid: String,
    },
}

fn required_attr<T: FromStr>(root: &Element, attr: &str, err: Error) -> Result<T, Error> {
    root.attr(attr)
        .and_then(|value| value.parse().ok())
        .ok_or(err)
}

pub fn parse_ibb(root: &Element) -> Result<IBB, Error> {
    if root.is("open", ns::IBB) {
        for _ in root.children() {
            return Err(Error::ParseError("Unknown child in open element."));
        }
        let block_size = required_attr(root, "block-size", Error::ParseError("Required attribute 'block-size' missing in open element."))?;
        let sid = required_attr(root, "sid", Error::ParseError("Required attribute 'sid' missing in open element."))?;
        let stanza = match root.attr("stanza") {
            Some(stanza) => stanza.parse()?,
            None => Default::default(),
        };
        Ok(IBB::Open {
            block_size: block_size,
            sid: sid,
            stanza: stanza
        })
    } else if root.is("data", ns::IBB) {
        for _ in root.children() {
            return Err(Error::ParseError("Unknown child in data element."));
        }
        let seq = required_attr(root, "seq", Error::ParseError("Required attribute 'seq' missing in data element."))?;
        let sid = required_attr(root, "sid", Error::ParseError("Required attribute 'sid' missing in data element."))?;
        let data = base64::decode(&root.text())?;
        Ok(IBB::Data {
            seq: seq,
            sid: sid,
            data: data
        })
    } else if root.is("close", ns::IBB) {
        let sid = required_attr(root, "sid", Error::ParseError("Required attribute 'sid' missing in data element."))?;
        for _ in root.children() {
            return Err(Error::ParseError("Unknown child in close element."));
        }
        Ok(IBB::Close {
            sid: sid,
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
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='3' sid='coucou'/>".parse().unwrap();
        let open = ibb::parse_ibb(&elem).unwrap();
        match open {
            ibb::IBB::Open { block_size, sid, stanza } => {
                assert_eq!(block_size, 3);
                assert_eq!(sid, "coucou");
                assert_eq!(stanza, ibb::Stanza::Iq);
            },
            _ => panic!(),
        }

        let elem: Element = "<data xmlns='http://jabber.org/protocol/ibb' seq='0' sid='coucou'>AAAA</data>".parse().unwrap();
        let data = ibb::parse_ibb(&elem).unwrap();
        match data {
            ibb::IBB::Data { seq, sid, data } => {
                assert_eq!(seq, 0);
                assert_eq!(sid, "coucou");
                assert_eq!(data, vec!(0, 0, 0));
            },
            _ => panic!(),
        }

        let elem: Element = "<close xmlns='http://jabber.org/protocol/ibb' sid='coucou'/>".parse().unwrap();
        let close = ibb::parse_ibb(&elem).unwrap();
        match close {
            ibb::IBB::Close { sid } => {
                assert_eq!(sid, "coucou");
            },
            _ => panic!(),
        }
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
    fn test_invalid_stanza() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = ibb::parse_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid 'stanza' attribute.");
    }
}
