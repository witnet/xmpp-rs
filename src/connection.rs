use transport::Transport;
use error::Error;
use ns;

use quick_xml::events::{Event as WriterEvent, BytesStart, BytesEnd};

pub trait Connection {
    type InitError;
    type CloseError;

    fn namespace() -> &'static str;

    fn init<T: Transport>(transport: &mut T, domain: &str, id: &str) -> Result<(), Self::InitError>;
    fn close<T: Transport>(transport: &mut T) -> Result<(), Self::CloseError>;
}

pub struct C2S;

impl Connection for C2S {
    type InitError = Error;
    type CloseError = Error;

    fn namespace() -> &'static str { ns::CLIENT }

    fn init<T: Transport>(transport: &mut T, domain: &str, id: &str) -> Result<(), Error> {
        let name = "stream:stream";
        let mut elem = BytesStart::borrowed(name.as_bytes(), name.len());
        elem.push_attribute(("to", domain));
        elem.push_attribute(("id", id));
        elem.push_attribute(("xmlns", ns::CLIENT));
        elem.push_attribute(("xmlns:stream", ns::STREAM));
        transport.write_event(WriterEvent::Start(elem))
    }

    fn close<T: Transport>(transport: &mut T) -> Result<(), Error> {
        let name = "stream:stream";
        let elem = BytesEnd::borrowed(name.as_bytes());
        transport.write_event(WriterEvent::End(elem))
    }
}

pub struct Component2S;

impl Connection for Component2S {
    type InitError = Error;
    type CloseError = Error;

    fn namespace() -> &'static str { ns::COMPONENT_ACCEPT }

    fn init<T: Transport>(transport: &mut T, domain: &str, id: &str) -> Result<(), Error> {
        let name = "stream:stream";
        let mut elem = BytesStart::borrowed(name.as_bytes(), name.len());
        elem.push_attribute(("to", domain));
        elem.push_attribute(("id", id));
        elem.push_attribute(("xmlns", ns::COMPONENT_ACCEPT));
        elem.push_attribute(("xmlns:stream", ns::STREAM));
        transport.write_event(WriterEvent::Start(elem))
    }

    fn close<T: Transport>(transport: &mut T) -> Result<(), Error> {
        let name = "stream:stream";
        let elem = BytesEnd::borrowed(name.as_bytes());
        transport.write_event(WriterEvent::End(elem))
    }
}
