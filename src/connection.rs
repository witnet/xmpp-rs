use transport::Transport;
use error::Error;
use ns;

use xml::writer::XmlEvent as WriterEvent;

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
        transport.write_event(WriterEvent::start_element("stream:stream")
                                          .attr("to", domain)
                                          .attr("id", id)
                                          .default_ns(ns::CLIENT)
                                          .ns("stream", ns::STREAM))?;
        Ok(())
    }

    fn close<T: Transport>(transport: &mut T) -> Result<(), Error> {
        transport.write_event(WriterEvent::end_element())?;
        Ok(())
    }
}
