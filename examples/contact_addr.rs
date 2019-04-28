use futures::{future, Sink, Stream};
use std::env::args;
use std::process::exit;
use tokio::runtime::current_thread::Runtime;
use tokio_xmpp::{Client, xmpp_codec::Packet};
use xmpp_parsers::{
    Element,
    Jid,
    TryFrom,
    ns,
    iq::{
        Iq,
        IqType,
    },
    disco::{
        DiscoInfoResult,
        DiscoInfoQuery,
    },
    server_info::ServerInfo,
};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 4 {
        println!("Usage: {} <jid> <password> <target>", args[0]);
        exit(1);
    }
    let jid = &args[1];
    let password = &args[2];
    let target = &args[3];

    // tokio_core context
    let mut rt = Runtime::new().unwrap();
    // Client instance
    let client = Client::new(jid, password).unwrap();

    // Make the two interfaces for sending and receiving independent
    // of each other so we can move one into a closure.
    let (mut sink, stream) = client.split();
    // Wrap sink in Option so that we can take() it for the send(self)
    // to consume and return it back when ready.
    let mut send = move |packet| {
        sink.start_send(packet).expect("start_send");
    };
    // Main loop, processes events
    let mut wait_for_stream_end = false;
    let done = stream.for_each(|event| {
        if wait_for_stream_end {
            /* Do Nothing. */
        } else if event.is_online() {
            println!("Online!");

            let target_jid: Jid = target.clone().parse().unwrap();
            let iq = make_disco_iq(target_jid);
            println!("Sending disco#info request to {}", target.clone());
            println!(">> {}", String::from(&iq));
            send(Packet::Stanza(iq));
        } else if let Some(stanza) = event.into_stanza() {
            if stanza.is("iq", "jabber:client") {
                let iq = Iq::try_from(stanza).unwrap();
                if let IqType::Result(Some(payload)) = iq.payload {
                    if payload.is("query", ns::DISCO_INFO) {
                        if let Ok(disco_info) = DiscoInfoResult::try_from(payload) {
                            for ext in disco_info.extensions {
                                if let Ok(server_info) = ServerInfo::try_from(ext) {
                                    print_server_info(server_info);
                                    wait_for_stream_end = true;
                                    send(Packet::StreamEnd);
                                }
                            }
                        }
                    }
                }
            }
        }

        Box::new(future::ok(()))
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

fn make_disco_iq(target: Jid) -> Element {
    Iq::from_get("disco", DiscoInfoQuery { node: None })
        .with_id(String::from("contact"))
        .with_to(target)
        .into()
}

fn convert_field(field: Vec<String>) -> String {
    field.iter()
    .fold((field.len(), String::new()), |(l, mut acc), s| {
        acc.push('<');
        acc.push_str(&s);
        acc.push('>');
        if l > 1 {
            acc.push(',');
            acc.push(' ');
        }
        (0, acc)
    }).1
}

fn print_server_info(server_info: ServerInfo) {
    if server_info.abuse.len() != 0 {
        println!("abuse: {}", convert_field(server_info.abuse));
    }
    if server_info.admin.len() != 0 {
        println!("admin: {}", convert_field(server_info.admin));
    }
    if server_info.feedback.len() != 0 {
        println!("feedback: {}", convert_field(server_info.feedback));
    }
    if server_info.sales.len() != 0 {
        println!("sales: {}", convert_field(server_info.sales));
    }
    if server_info.security.len() != 0 {
        println!("security: {}", convert_field(server_info.security));
    }
    if server_info.support.len() != 0 {
        println!("support: {}", convert_field(server_info.support));
    }
}
