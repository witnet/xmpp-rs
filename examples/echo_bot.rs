use futures::{future, Future, Sink, Stream};
use std::convert::TryFrom;
use std::env::args;
use std::process::exit;
use tokio::runtime::current_thread::Runtime;
use tokio_xmpp::{Client, Packet};
use xmpp_parsers::{Jid, Element};
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

    // Create outgoing pipe
    let (mut tx, rx) = futures::unsync::mpsc::unbounded();
    rt.spawn(
        rx.forward(
            sink.sink_map_err(|_| panic!("Pipe"))
        )
            .map(|(rx, mut sink)| {
                drop(rx);
                let _ = sink.close();
            })
            .map_err(|e| {
                panic!("Send error: {:?}", e);
            })
    );

    // Main loop, processes events
    let mut wait_for_stream_end = false;
    let done = stream.for_each(move |event| {
        if wait_for_stream_end {
            /* Do nothing */
        } else if event.is_online() {
            println!("Online!");

            let presence = make_presence();
            tx.start_send(Packet::Stanza(presence)).unwrap();
        } else if let Some(message) = event
            .into_stanza()
            .and_then(|stanza| Message::try_from(stanza).ok())
        {
            match (message.from, message.bodies.get("")) {
                (Some(ref from), Some(ref body)) if body.0 == "die" => {
                    println!("Secret die command triggered by {}", from.clone());
                    wait_for_stream_end = true;
                    tx.start_send(Packet::StreamEnd).unwrap();
                }
                (Some(ref from), Some(ref body)) => {
                    if message.type_ != MessageType::Error {
                        // This is a message we'll echo
                        let reply = make_reply(from.clone(), &body.0);
                        tx.start_send(Packet::Stanza(reply)).unwrap();
                    }
                }
                _ => {}
            }
        }

        future::ok(())
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
    presence.show = Some(PresenceShow::Chat);
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
