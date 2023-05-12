use std::{error::Error, net::TcpListener, thread};

use crate::service::domain;
use crate::service::domain::events;

use nostr::{ClientMessage, Event};
use tungstenite;

pub fn listen_and_serve() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();
                match handle_message(msg) {
                    Ok(_) => continue,
                    Err(err) => {
                        println!("{}", err);
                        break;
                    }
                };
            }
        });
    }
}

fn handle_message(msg: tungstenite::Message) -> Result<(), Box<dyn Error>> {
    if !(msg.is_ping() || msg.is_pong()) {
        return Ok(());
    }

    let msg_text = match msg.into_text() {
        Ok(value) => value,
        Err(err) => return Err(Box::new(err)),
    };

    let client_message = match nostr::ClientMessage::from_json(msg_text) {
        Ok(value) => value,
        Err(err) => return Err(Box::new(err)),
    };

    println!("Received client message: {}", client_message.as_json());

    match client_message {
        ClientMessage::Event(event) => {
            // todo check if right event?

            let registration_event_content: events::RegistrationEventContent =
                serde_json::from_str(&event.content)?;
            let apns_token = domain::APNSToken::new(registration_event_content.apns_token)?;
            let relays: Result<Vec<domain::RelayAddress>, Box<dyn Error>> =
                registration_event_content
                    .relays
                    .iter()
                    .map(|v| domain::RelayAddress::new(v.to_string()))
                    .collect();
            let locale = domain::Locale::new(registration_event_content.locale)?;

            let registration = domain::Registration::new(apns_token, relays?, locale)?;

            // todo use registration

            return Ok(());
        }
        _ => return Ok(()),
    }
}
