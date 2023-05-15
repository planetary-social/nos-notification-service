//use std::sync::{Arc};
use std::{error::Error, net::TcpListener};

use crate::service::app;
use crate::service::app::commands::Register;
use crate::service::domain;
use crate::service::domain::events;

use crossbeam::thread;
use nostr::ClientMessage;
use tungstenite;

pub struct Server<'a> {
    app: &'a app::Application<'a>,
}

impl<'a> Server<'_> {
    pub fn new(app: &'a app::Application) -> Server<'a> {
        return Server { app };
    }

    pub fn listen_and_serve(&self) {
        let address = String::from("127.0.0.1:7878");
        let listener = TcpListener::bind(address).unwrap();

        thread::scope(|s| {
            for stream in listener.incoming() {
                //let shared_self = Arc::new(self);
                s.spawn(|_| {
                    let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
                    loop {
                        let msg = websocket.read_message().unwrap();
                        match self.handle_message(msg) {
                            Ok(_) => continue,
                            Err(err) => {
                                println!("error handling the received message: {}", err);
                                break;
                            }
                        };
                    }
                });
            }
        })
        .unwrap();
    }

    fn handle_message(&self, msg: tungstenite::Message) -> Result<(), Box<dyn Error>> {
        if msg.is_ping() || msg.is_pong() {
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
                let cmd = Register { registration };
                return self.app.commands.register.handle(&cmd);
            }
            _ => return Ok(()),
        }
    }
}
