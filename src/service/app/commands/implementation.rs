use crate::app::commands;

use std::error::Error;

pub struct RegisterHandler {}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {}
    }
}

impl commands::RegisterHandler for RegisterHandler {
    fn handle(&self, cmd: &commands::Register) -> Result<(), Box<dyn Error>> {
        println!("apns_token {}", cmd.registration.apns_token().as_ref());
        Ok(())
    }
}
