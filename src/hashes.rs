use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone, PartialEq)]
pub struct Hash {
    pub algo: String,
    pub hash: String,
}

pub fn parse_hash(root: &Element) -> Result<Hash, Error> {
    if !root.is("hash", ns::HASHES) {
        return Err(Error::ParseError("This is not a hash element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in hash element."));
    }
    let algo = root.attr("algo").ok_or(Error::ParseError("Mandatory argument 'algo' not present in hash element."))?.to_owned();
    let hash = match root.text().as_ref() {
        "" => return Err(Error::ParseError("Hash element shouldn’t be empty.")),
        text => text.to_owned(),
    };
    Ok(Hash {
        algo: algo,
        hash: hash,
    })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use hashes;

    #[test]
    fn test_simple() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>".parse().unwrap();
        let hash = hashes::parse_hash(&elem).unwrap();
        assert_eq!(hash.algo, "sha-256");
        assert_eq!(hash.hash, "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=");
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = hashes::parse_hash(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a hash element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2'><coucou/></hash>".parse().unwrap();
        let error = hashes::parse_hash(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in hash element.");
    }
}
