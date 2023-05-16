use crate::errors::Result;
use crate::service::app::common;
use crate::service::domain;
use sqlite;
use std::borrow::Borrow;
use std::sync::Mutex;

pub type AdaptersFactoryFn = fn(&sqlite::Connection) -> common::Adapters;

pub struct TransactionProvider {
    conn: Mutex<sqlite::Connection>,
    factory_fn: Box<AdaptersFactoryFn>,
}

impl<'a> TransactionProvider {
    pub fn new(
        conn: sqlite::Connection,
        factory_fn: Box<AdaptersFactoryFn>,
    ) -> TransactionProvider {
        return TransactionProvider {
            conn: Mutex::new(conn),
            factory_fn,
        };
    }
}

impl common::TransactionProvider for TransactionProvider {
    fn transact(&self, f: &common::TransactionFn) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("BEGIN TRANSACTION")?;

        let adapters = (self.factory_fn)(&conn);
        match f(adapters) {
            Ok(()) => {
                conn.execute("COMMIT TRANSACTION")?;
                return Ok(());
            }
            Err(err) => {
                conn.execute("ROLLBACK TRANSACTION")?;
                return Err(err);
            }
        }
    }
}

pub struct RegistrationRepository<'a> {
    conn: &'a sqlite::Connection,
}

impl RegistrationRepository<'_> {
    pub fn new(conn: &sqlite::Connection) -> RegistrationRepository {
        return RegistrationRepository { conn };
    }
}

impl common::RegistrationRepository for RegistrationRepository<'_> {
    fn save(&self, registration: domain::Registration) -> Result<()> {
        return Err("not implemented")?;
    }
}
