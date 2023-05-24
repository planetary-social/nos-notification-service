use crate::errors::Result;
use crate::service::domain;

pub type TransactionFn<A> = fn(adapters: A) -> Result<()>;

pub trait TransactionProvider<A> {
    fn transact(&self, f: TransactionFn<A>) -> Result<()>;
}

pub trait RegistrationRepository {
    fn save(&self, registration: domain::Registration) -> Result<()>;
}

pub struct Adapters<T>
where
    T: RegistrationRepository,
{
    pub registrations: T,
}
