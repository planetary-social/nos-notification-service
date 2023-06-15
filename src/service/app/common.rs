use crate::errors::Result;
use crate::service::domain;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Transaction {
    fn adapters(&self) -> Adapters;
    fn commit(&self) -> Result<()>;
}

pub trait TransactionProvider {
    fn start_transaction(&self) -> Result<Box<dyn Transaction>>;
}

#[derive(Clone)]
pub struct Adapters {
    pub registrations: Rc<RefCell<Box<dyn RegistrationRepository>>>,
    pub events: Rc<RefCell<Box<dyn EventRepository>>>,
}

impl Adapters {
    pub fn new(
        registrations: Box<dyn RegistrationRepository>,
        events: Box<dyn EventRepository>,
    ) -> Adapters {
        Adapters {
            registrations: Rc::new(RefCell::new(registrations)),
            events: Rc::new(RefCell::new(events)),
        }
    }
}

pub trait RegistrationRepository {
    fn save(&self, registration: &domain::Registration) -> Result<()>;
    fn get_relays(&self) -> Result<Vec<domain::RelayAddress>>;
    fn get_pub_keys(&self, relay: &domain::RelayAddress) -> Result<Vec<PubKeyInfo>>;
}

pub trait EventRepository {
    fn save_event(&self);
}

#[derive(Debug, Eq, PartialEq)]
pub struct PubKeyInfo {
    pub_key: domain::PubKey,
    //last_event: Option<time::Instant>,
}

impl PubKeyInfo {
    pub fn new(pub_key: domain::PubKey) -> Self {
        Self { pub_key }
    }

    pub fn pub_key(&self) -> &domain::PubKey {
        &self.pub_key
    }
}
