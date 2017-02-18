use jid::Jid;

pub struct XmppClient {
    transport: SslTransport,
}

impl XmppClient {
    pub fn connect(jid: Jid) -> XmppClient {
        unimplemented!();
    }
}
