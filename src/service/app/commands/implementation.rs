use crate::app::commands;
use crate::errors::Result;

pub struct RegisterHandler {}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {}
    }
}

impl commands::RegisterHandler for RegisterHandler {
    fn handle(&self, cmd: &commands::Register) -> Result<()> {
        println!("apns_token {}", cmd.registration.apns_token().as_ref());
        Ok(())
    }
}
