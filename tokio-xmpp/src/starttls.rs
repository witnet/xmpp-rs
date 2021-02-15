use futures::{sink::SinkExt, stream::StreamExt};
#[cfg(feature = "tls-rust")]
use idna;
#[cfg(feature = "tls-native")]
use native_tls::TlsConnector as NativeTlsConnector;
#[cfg(feature = "tls-rust")]
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "tls-native")]
use tokio_native_tls::{TlsConnector, TlsStream};
#[cfg(feature = "tls-rust")]
use tokio_rustls::{client::TlsStream, rustls::ClientConfig, TlsConnector};
#[cfg(feature = "tls-rust")]
use webpki::DNSNameRef;
use xmpp_parsers::{ns, Element};

use crate::xmpp_codec::Packet;
use crate::xmpp_stream::XMPPStream;
use crate::{Error, ProtocolError};

#[cfg(feature = "tls-native")]
async fn get_tls_stream<S: AsyncRead + AsyncWrite + Unpin>(
    xmpp_stream: XMPPStream<S>,
) -> Result<TlsStream<S>, Error> {
    let domain = &xmpp_stream.jid.clone().domain();
    let stream = xmpp_stream.into_inner();
    let tls_stream = TlsConnector::from(NativeTlsConnector::builder().build().unwrap())
        .connect(&domain, stream)
        .await?;
    Ok(tls_stream)
}

#[cfg(feature = "tls-rust")]
async fn get_tls_stream<S: AsyncRead + AsyncWrite + Unpin>(
    xmpp_stream: XMPPStream<S>,
) -> Result<TlsStream<S>, Error> {
    let domain = &xmpp_stream.jid.clone().domain();
    let ascii_domain = idna::domain_to_ascii(domain).map_err(|_| Error::Idna)?;
    let domain = DNSNameRef::try_from_ascii_str(&ascii_domain).unwrap();
    let stream = xmpp_stream.into_inner();
    let tls_stream = TlsConnector::from(Arc::new(ClientConfig::new()))
        .connect(domain, stream)
        .await?;
    Ok(tls_stream)
}

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

    get_tls_stream(xmpp_stream).await
}
