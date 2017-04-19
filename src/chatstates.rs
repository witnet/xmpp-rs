use minidom::Element;

use error::Error;
use super::MessagePayload;

use ns::CHATSTATES_NS;

#[derive(Debug)]
pub enum ChatState {
    Active,
    Composing,
    Gone,
    Inactive,
    Paused,
}

impl MessagePayload for ChatState {}

pub fn parse_chatstate(root: &Element) -> Result<ChatState, Error> {
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in chatstate element."));
    }
    if root.is("active", CHATSTATES_NS) {
        Ok(ChatState::Active)
    } else if root.is("composing", CHATSTATES_NS) {
        Ok(ChatState::Composing)
    } else if root.is("gone", CHATSTATES_NS) {
        Ok(ChatState::Gone)
    } else if root.is("inactive", CHATSTATES_NS) {
        Ok(ChatState::Inactive)
    } else if root.is("paused", CHATSTATES_NS) {
        Ok(ChatState::Paused)
    } else {
        Err(Error::ParseError("Unknown chatstate element."))
    }
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
