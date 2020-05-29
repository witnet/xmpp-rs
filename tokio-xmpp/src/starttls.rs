use futures::{sink::SinkExt, stream::StreamExt};
use native_tls::TlsConnector as NativeTlsConnector;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tls::{TlsConnector, TlsStream};
use xmpp_parsers::{ns, Element};

use crate::xmpp_codec::Packet;
use crate::xmpp_stream::XMPPStream;
use crate::{Error, ProtocolError};

/// Performs `<starttls/>` on an XMPPStream and returns a binary
/// TlsStream.
pub async fn starttls<S: AsyncRead + AsyncWrite + Unpin>(
    mut xmpp_stream: XMPPStream<S>,
) -> Result<TlsStream<S>, Error> {
    let nonza = Element::builder("starttls", ns::TLS).build();
    let packet = Packet::Stanza(nonza);
    xmpp_stream.send(packet).await?;

    loop {
        match xmpp_stream.next().await {
            Some(Ok(Packet::Stanza(ref stanza))) if stanza.name() == "proceed" => break,
            Some(Ok(Packet::Text(_))) => {}
            Some(Err(e)) => return Err(e.into()),
            _ => {
                return Err(ProtocolError::NoTls.into());
            }
        }
    }

    let domain = xmpp_stream.jid.clone().domain();
    let stream = xmpp_stream.into_inner();
    let tls_stream = TlsConnector::from(NativeTlsConnector::builder().build().unwrap())
        .connect(&domain, stream)
        .await?;

    Ok(tls_stream)
}
