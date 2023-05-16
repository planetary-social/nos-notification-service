use crate::errors::Result;
use crate::service::domain;

pub type TransactionFn = fn(adapters: Adapters) -> Result<()>;

pub trait TransactionProvider {
    fn transact(&self, f: &TransactionFn) -> Result<()>;
}

pub trait RegistrationRepository {
    fn save(&self, registration: domain::Registration) -> Result<()>;
}

pub struct Adapters<'a> {
    pub registrations: Box<dyn RegistrationRepository + 'a>,
}
