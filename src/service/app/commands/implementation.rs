use crate::app::commands;

use std::error::Error;

pub struct RegisterHandler {}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        return RegisterHandler {};
    }
}

impl commands::RegisterHandler for RegisterHandler {
    fn handle(&self, cmd: &commands::Register) -> Result<(), Box<dyn Error>> {
        let apns_token: String = cmd.registration.apns_token().into();
        println!("apns_token {}", apns_token);
        return Ok(());
    }
}
