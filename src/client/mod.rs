use futures::{done, Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use idna;
use xmpp_parsers::{Jid, JidParseError};
use sasl::common::{ChannelBinding, Credentials};
use std::mem::replace;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_tls::TlsStream;

use super::event::Event;
use super::happy_eyeballs::Connecter;
use super::starttls::{StartTlsClient, NS_XMPP_TLS};
use super::xmpp_codec::Packet;
use super::xmpp_stream;
use super::{Error, ProtocolError};

mod auth;
use self::auth::ClientAuth;
mod bind;
use self::bind::ClientBind;

/// XMPP client connection and state
pub struct Client {
    /// The client's current Jabber-Id
    pub jid: Jid,
    state: ClientState,
}

type XMPPStream = xmpp_stream::XMPPStream<TlsStream<TcpStream>>;
const NS_JABBER_CLIENT: &str = "jabber:client";

enum ClientState {
    Invalid,
    Disconnected,
    Connecting(Box<dyn Future<Item = XMPPStream, Error = Error>>),
    Connected(XMPPStream),
}

impl Client {
    /// Start a new XMPP client
    ///
    /// Start polling the returned instance so that it will connect
    /// and yield events.
    pub fn new(jid: &str, password: &str) -> Result<Self, JidParseError> {
        let jid = Jid::from_str(jid)?;
        let client = Self::new_with_jid(jid, password);
        Ok(client)
    }

    /// Start a new client given that the JID is already parsed.
    pub fn new_with_jid(jid: Jid, password: &str) -> Self {
        let password = password.to_owned();
        let connect = Self::make_connect(jid.clone(), password.clone());
        let client = Client {
            jid,
            state: ClientState::Connecting(Box::new(connect)),
        };
        client
    }

    fn make_connect(jid: Jid, password: String) -> impl Future<Item = XMPPStream, Error = Error> {
        let username = jid.clone().node().unwrap();
        let jid1 = jid.clone();
        let jid2 = jid.clone();
        let password = password;
        done(idna::domain_to_ascii(&jid.domain()))
            .map_err(|_| Error::Idna)
            .and_then(|domain| {
                done(Connecter::from_lookup(
                    &domain,
                    Some("_xmpp-client._tcp"),
                    5222,
                ))
            })
            .flatten()
            .and_then(move |tcp_stream| {
                xmpp_stream::XMPPStream::start(tcp_stream, jid1, NS_JABBER_CLIENT.to_owned())
            })
            .and_then(|xmpp_stream| {
                if Self::can_starttls(&xmpp_stream) {
                    Ok(Self::starttls(xmpp_stream))
                } else {
                    Err(Error::Protocol(ProtocolError::NoTls))
                }
            })
            .flatten()
            .and_then(|tls_stream| XMPPStream::start(tls_stream, jid2, NS_JABBER_CLIENT.to_owned()))
            .and_then(
                move |xmpp_stream| done(Self::auth(xmpp_stream, username, password)), // TODO: flatten?
            )
            .and_then(|auth| auth)
            .and_then(|xmpp_stream| Self::bind(xmpp_stream))
            .and_then(|xmpp_stream| {
                // println!("Bound to {}", xmpp_stream.jid);
                Ok(xmpp_stream)
            })
    }

    fn can_starttls<S>(stream: &xmpp_stream::XMPPStream<S>) -> bool {
        stream
            .stream_features
            .get_child("starttls", NS_XMPP_TLS)
            .is_some()
    }

    fn starttls<S: AsyncRead + AsyncWrite>(
        stream: xmpp_stream::XMPPStream<S>,
    ) -> StartTlsClient<S> {
        StartTlsClient::from_stream(stream)
    }

    fn auth<S: AsyncRead + AsyncWrite + 'static>(
        stream: xmpp_stream::XMPPStream<S>,
        username: String,
        password: String,
    ) -> Result<ClientAuth<S>, Error> {
        let creds = Credentials::default()
            .with_username(username)
            .with_password(password)
            .with_channel_binding(ChannelBinding::None);
        ClientAuth::new(stream, creds)
    }

    fn bind<S: AsyncWrite>(stream: xmpp_stream::XMPPStream<S>) -> ClientBind<S> {
        ClientBind::new(stream)
    }
}

impl Stream for Client {
    type Item = Event;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let state = replace(&mut self.state, ClientState::Invalid);

        match state {
            ClientState::Invalid => Err(Error::InvalidState),
            ClientState::Disconnected => Ok(Async::Ready(None)),
            ClientState::Connecting(mut connect) => match connect.poll() {
                Ok(Async::Ready(stream)) => {
                    self.state = ClientState::Connected(stream);
                    Ok(Async::Ready(Some(Event::Online)))
                }
                Ok(Async::NotReady) => {
                    self.state = ClientState::Connecting(connect);
                    Ok(Async::NotReady)
                }
                Err(e) => Err(e),
            },
            ClientState::Connected(mut stream) => {
                // Poll sink
                match stream.poll_complete() {
                    Ok(Async::NotReady) => (),
                    Ok(Async::Ready(())) => (),
                    Err(e) => return Err(e)?,
                };

                // Poll stream
                match stream.poll() {
                    Ok(Async::Ready(None)) => {
                        // EOF
                        self.state = ClientState::Disconnected;
                        Ok(Async::Ready(Some(Event::Disconnected)))
                    }
                    Ok(Async::Ready(Some(Packet::Stanza(stanza)))) => {
                        // Receive stanza
                        self.state = ClientState::Connected(stream);
                        Ok(Async::Ready(Some(Event::Stanza(stanza))))
                    }
                    Ok(Async::Ready(Some(Packet::Text(_)))) => {
                        // Ignore text between stanzas
                        Ok(Async::NotReady)
                    }
                    Ok(Async::Ready(Some(Packet::StreamStart(_)))) => {
                        // <stream:stream>
                        Err(ProtocolError::InvalidStreamStart.into())
                    }
                    Ok(Async::Ready(Some(Packet::StreamEnd))) => {
                        // End of stream: </stream:stream>
                        Ok(Async::Ready(None))
                    }
                    Ok(Async::NotReady) => {
                        // Try again later
                        self.state = ClientState::Connected(stream);
                        Ok(Async::NotReady)
                    }
                    Err(e) => Err(e)?,
                }
            }
        }
    }
}

impl Sink for Client {
    type SinkItem = Packet;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self.state {
            ClientState::Connected(ref mut stream) =>
                Ok(stream.start_send(item)?),
            _ =>
                Ok(AsyncSink::NotReady(item)),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self.state {
            ClientState::Connected(ref mut stream) => stream.poll_complete().map_err(|e| e.into()),
            _ => Ok(Async::Ready(())),
        }
    }

    /// This closes the inner TCP stream.
    ///
    /// To synchronize your shutdown with the server side, you should
    /// first send `Packet::StreamEnd` and wait for the end of the
    /// incoming stream before closing the connection.
    fn close(&mut self) -> Poll<(), Self::SinkError> {
        match self.state {
            ClientState::Connected(ref mut stream) =>
                stream.close()
                .map_err(|e| e.into()),
            _ =>
                Ok(Async::Ready(())),
        }
    }
}
