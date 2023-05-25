pub mod implementation;

use crate::errors::Result;
use crate::service::domain::Registration;

pub struct Register {
    pub registration: Registration,
}

pub trait RegisterHandler {
    fn handle(&self, cmd: &Register) -> Result<()>;
}
