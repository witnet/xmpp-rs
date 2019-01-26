use futures::{future, Future, Sink, Stream};
use std::env::args;
use std::process::exit;
use tokio::runtime::current_thread::Runtime;
use tokio_xmpp::Client;
use xmpp_parsers::{Jid, Element, TryFrom};
use xmpp_parsers::message::{Body, Message, MessageType};
use xmpp_parsers::presence::{Presence, Show as PresenceShow, Type as PresenceType};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <jid> <password>", args[0]);
        exit(1);
    }
    let jid = &args[1];
    let password = &args[2];

    // tokio_core context
    let mut rt = Runtime::new().unwrap();
    // Client instance
    let client = Client::new(jid, password).unwrap();

    // Make the two interfaces for sending and receiving independent
    // of each other so we can move one into a closure.
    let (sink, stream) = client.split();
    let mut sink_state = Some(sink);
    // Main loop, processes events
    let done = stream.for_each(move |event| {
        let mut sink_future = None;

        if event.is_online() {
            println!("Online!");

            let presence = make_presence();
            let sink = sink_state.take().unwrap();
            sink_future = Some(Box::new(sink.send(presence)));
        } else if let Some(message) = event
            .into_stanza()
            .and_then(|stanza| Message::try_from(stanza).ok())
        {
            match (message.from, message.bodies.get("")) {
                (Some(ref from), Some(ref body)) if body.0 == "die" => {
                    println!("Secret die command triggered by {}", from);
                    let sink = sink_state.as_mut().unwrap();
                    sink.close().expect("close");
                }
                (Some(ref from), Some(ref body)) => {
                    if message.type_ != MessageType::Error {
                        // This is a message we'll echo
                        let reply = make_reply(from.clone(), &body.0);
                        let sink = sink_state.take().unwrap();
                        sink_future = Some(Box::new(sink.send(reply)));
                    }
                }
                _ => {}
            }
        };

        sink_future
            .map(|future| {
                let wait_send: Box<Future<Item = (), Error = tokio_xmpp::Error>> =
                    Box::new(future
                             .map(|sink| {
                                 sink_state = Some(sink);
                             }));
                wait_send
            })
            .unwrap_or_else(|| Box::new(future::ok(())))
    });

    // Start polling `done`
    match rt.block_on(done) {
        Ok(_) => (),
        Err(e) => {
            println!("Fatal: {}", e);
            ()
        }
    }
}

// Construct a <presence/>
fn make_presence() -> Element {
    let mut presence = Presence::new(PresenceType::None);
    presence.show = PresenceShow::Chat;
    presence
        .statuses
        .insert(String::from("en"), String::from("Echoing messages."));
    presence.into()
}

// Construct a chat <message/>
fn make_reply(to: Jid, body: &str) -> Element {
    let mut message = Message::new(Some(to));
    message.bodies.insert(String::new(), Body(body.to_owned()));
    message.into()
}
