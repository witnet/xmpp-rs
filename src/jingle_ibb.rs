use std::str::FromStr;

use minidom::Element;

use error::Error;

use ns;

use ibb::Stanza;

#[derive(Debug, Clone)]
pub struct Transport {
    block_size: u16,
    sid: String,
    stanza: Stanza,
}

fn optional_attr<T: FromStr>(root: &Element, attr: &str) -> Option<T> {
    root.attr(attr)
        .and_then(|value| value.parse().ok())
}

fn required_attr<T: FromStr>(root: &Element, attr: &str, err: Error) -> Result<T, Error> {
    optional_attr(root, attr).ok_or(err)
}

pub fn parse_jingle_ibb(root: &Element) -> Result<Transport, Error> {
    if root.is("transport", ns::JINGLE_IBB) {
        for _ in root.children() {
            return Err(Error::ParseError("Unknown child in JingleIBB element."));
        }
        let block_size = required_attr(root, "block-size", Error::ParseError("Required attribute 'block-size' missing in JingleIBB element."))?;
        let sid = required_attr(root, "sid", Error::ParseError("Required attribute 'sid' missing in JingleIBB element."))?;
        let stanza = root.attr("stanza")
                         .unwrap_or("iq")
                         .parse()?;
        Ok(Transport {
            block_size: block_size,
            sid: sid,
            stanza: stanza
        })
    } else {
        Err(Error::ParseError("This is not an JingleIBB element."))
    }
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use ibb;
    use jingle_ibb;

    #[test]
    fn test_simple() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='3' sid='coucou'/>".parse().unwrap();
        let transport = jingle_ibb::parse_jingle_ibb(&elem).unwrap();
        assert_eq!(transport.block_size, 3);
        assert_eq!(transport.sid, "coucou");
        assert_eq!(transport.stanza, ibb::Stanza::Iq);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1'/>".parse().unwrap();
        let error = jingle_ibb::parse_jingle_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing in JingleIBB element.");

        // TODO: maybe make a better error message here.
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='-5'/>".parse().unwrap();
        let error = jingle_ibb::parse_jingle_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing in JingleIBB element.");

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128'/>".parse().unwrap();
        let error = jingle_ibb::parse_jingle_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing in JingleIBB element.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = jingle_ibb::parse_jingle_ibb(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid 'stanza' attribute.");
    }
}
