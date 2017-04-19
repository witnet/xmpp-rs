use minidom::Element;

use error::Error;

use ns::RECEIPTS_NS;

#[derive(Debug)]
pub enum Receipt {
    Request,
    Received(String),
}

pub fn parse_receipt(root: &Element) -> Result<Receipt, Error> {
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in receipt element."));
    }
    if root.is("request", RECEIPTS_NS) {
        Ok(Receipt::Request)
    } else if root.is("received", RECEIPTS_NS) {
        let id = root.attr("id").unwrap_or("").to_owned();
        Ok(Receipt::Received(id))
    } else {
        Err(Error::ParseError("This is not a receipt element."))
    }
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    //use error::Error;
    use receipts;

    #[test]
    fn test_simple() {
        let elem: Element = "<request xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();

        let elem: Element = "<received xmlns='urn:xmpp:receipts' id='coucou'/>".parse().unwrap();
        receipts::parse_receipt(&elem).unwrap();
    }
}
