use crate::errors::Result;
use crate::service::app;
use crate::service::app::commands::Register;
use crate::service::domain;
use crate::service::domain::events;
use crossbeam::thread;
use nostr::ClientMessage;
use std::net::TcpListener;
use tungstenite;

pub struct Server<'a> {
    app: &'a app::Application<'a>,
}

impl<'a> Server<'_> {
    pub fn new(app: &'a app::Application) -> Server<'a> {
        Server { app }
    }

    pub fn listen_and_serve(&self) {
        let address = String::from("127.0.0.1:7878");
        let listener = TcpListener::bind(address).unwrap();

        thread::scope(|s| {
            for stream in listener.incoming() {
                s.spawn(|_| {
                    let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
                    loop {
                        let msg = websocket.read_message().unwrap();
                        if let Err(err) = self.handle_message(msg) {
                            println!("error handling the received message: {err}");
                            break;
                        }
                    }
                });
            }
        })
        .unwrap();
    }

    fn handle_message(&self, msg: tungstenite::Message) -> Result<()> {
        if msg.is_ping() || msg.is_pong() {
            return Ok(());
        }

        let msg_text = msg.into_text()?;
        let client_message = nostr::ClientMessage::from_json(msg_text)?;

        match client_message {
            ClientMessage::Event(event) => {
                // todo check if right event?

                let registration_event_content: events::RegistrationEventContent =
                    serde_json::from_str(&event.content)?;
                let pub_key = domain::PubKey::new(event.pubkey);
                let apns_token = domain::APNSToken::new(registration_event_content.apns_token)?;
                let relays: Result<Vec<domain::RelayAddress>> = registration_event_content
                    .relays
                    .iter()
                    .map(|v| domain::RelayAddress::new(v.to_string()))
                    .collect();
                let locale = domain::Locale::new(registration_event_content.locale)?;

                let registration = domain::Registration::new(pub_key, apns_token, relays?, locale)?;
                let cmd = Register { registration };
                self.app.commands.register.handle(&cmd)
            }
            _ => Ok(()),
        }
    }
}
