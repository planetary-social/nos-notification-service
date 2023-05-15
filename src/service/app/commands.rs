pub mod implementation;

use std::error::Error;

use crate::service::domain::Registration;

pub struct Register {
    pub registration: Registration,
}

pub trait RegisterHandler {
    fn handle(&self, cmd: &Register) -> Result<(), Box<dyn Error>>;
}
