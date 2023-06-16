use crate::errors::Result;
use crate::migrations;
use crate::service::app::common;
use crate::service::domain;
use sqlite;
use sqlite::State;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone)]
pub struct TransactionProvider {
    pool: SqliteConnectionPool,
}

impl TransactionProvider {
    pub fn new(pool: SqliteConnectionPool) -> TransactionProvider {
        TransactionProvider { pool }
    }

    fn new_adapters<'a>(&self, conn: &'a SqliteConnection<'a>) -> common::Adapters<'a> {
        let registrations = Box::new(RegistrationRepository::new(conn));
        let events = Box::new(EventRepository::new(conn));
        common::Adapters::new(registrations, events)
    }
}

impl common::TransactionProvider for TransactionProvider {
    fn start_transaction<'a>(&'a self) -> Result<Box<dyn common::Transaction + 'a>> {
        let conn = self.pool.get();
        let adapters = self.new_adapters(&conn);
        let transaction = Transaction::new(conn, adapters)?;
        Ok(Box::new(transaction))
    }
}

struct Transaction<'a> {
    conn: SqliteConnection<'a>,
    adapters: Rc<common::Adapters<'a>>,
    commited: bool,
}

impl<'a> Transaction<'a> {
    fn new(conn: SqliteConnection<'a>, adapters: common::Adapters<'a>) -> Result<Transaction<'a>> {
        conn.0.execute("BEGIN TRANSACTION")?;

        Ok(
            Self {
            conn,
            adapters: Rc::new(adapters),
            commited: false,
        }
        )
    }
}

impl<'a> common::Transaction<'a> for Transaction<'a> {
    fn adapters(&self) -> Rc<common::Adapters<'a>> {
        let adapters = self.adapters.clone();
        adapters
    }

    fn commit(&mut self) -> Result<()> {
        self.conn.0.execute("COMMIT TRANSACTION")?;
        self.commited = true;
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.commited {
            if let Err(err) = self.conn.0.execute("ROLLBACK TRANSACTION") {
                dbg!("Error rolling back the transaction: {}", err);
            };
        }
    }
}

pub struct RegistrationRepository<'a> {
    conn: &'a SqliteConnection<'a>,
}

impl<'a> RegistrationRepository<'a> {
    pub fn new(conn: &'a SqliteConnection<'a>) -> RegistrationRepository<'a> {
        RegistrationRepository { conn }
    }
}

impl<'a> common::RegistrationRepository for RegistrationRepository<'a> {
    fn save(&self, registration: &domain::Registration) -> Result<()> {
        let hex_public_key = registration.pub_key().hex();

        let mut statement = self.conn.0.prepare(
            "INSERT OR REPLACE INTO
            registration(public_key, apns_token, locale)
            VALUES (:public_key, :apns_token, :locale)
        ",
        )?;
        statement.bind((":public_key", hex_public_key.as_str()))?;
        statement.bind((":apns_token", registration.apns_token().as_ref()))?;
        statement.bind((":locale", registration.locale().as_ref()))?;
        statement.next()?;

        let mut statement = self
            .conn
            .0
            .prepare("DELETE FROM relays WHERE public_key=:public_key")?;
        statement.bind((":public_key", hex_public_key.as_str()))?;
        statement.next()?;

        for address in registration.relays() {
            let mut statement = self.conn.0.prepare(
                "INSERT INTO relays (public_key, address) VALUES (:public_key, :address)",
            )?;
            statement.bind((":public_key", hex_public_key.as_str()))?;
            statement.bind((":address", address.as_ref()))?;
            statement.next()?;
        }

        Ok(())
    }

    fn get_relays(&self) -> Result<Vec<domain::RelayAddress>> {
        let query = "SELECT address FROM relays GROUP BY address";
        let mut statement = self.conn.0.prepare(query)?;

        let mut relay_addresses = Vec::new();

        while let Ok(State::Row) = statement.next() {
            let address_string = statement.read::<String, _>("address")?;
            let relay_address = domain::RelayAddress::new(address_string)?;
            relay_addresses.push(relay_address);
        }

        Ok(relay_addresses)
    }

    fn get_pub_keys(&self, address: &domain::RelayAddress) -> Result<Vec<common::PubKeyInfo>> {
        let query = "SELECT public_key FROM relays WHERE address = :address";
        let mut statement = self.conn.0.prepare(query)?;
        statement.bind((":address", address.as_ref()))?;

        let mut results = Vec::new();

        while let Ok(State::Row) = statement.next() {
            let public_key_string = statement.read::<String, _>("public_key")?;
            let pub_key = domain::PubKey::new_from_hex(public_key_string.as_ref())?;
            let pub_key_info = common::PubKeyInfo::new(pub_key);
            results.push(pub_key_info);
        }

        Ok(results)
    }
}

pub struct RegistrationRepositoryMigration0001 {
    pool: SqliteConnectionPool,
}

impl RegistrationRepositoryMigration0001 {
    pub fn new(pool: SqliteConnectionPool) -> RegistrationRepositoryMigration0001 {
        RegistrationRepositoryMigration0001 { pool }
    }
}

impl migrations::MigrationCallable for RegistrationRepositoryMigration0001 {
    fn run(&self) -> Result<()> {
        let conn = self.pool.get();

        conn.0.execute(
            "CREATE TABLE registration (
              public_key TEXT,
              apns_token TEXT,
              locale TEXT,
              PRIMARY KEY (public_key)
             )",
        )?;

        conn.0.execute(
            "CREATE TABLE relays (
              public_key TEXT,
              address TEXT,
              PRIMARY KEY (public_key, address),
              FOREIGN KEY (public_key) REFERENCES registration(public_key) ON DELETE CASCADE
             )",
        )?;
        Ok(())
    }
}

pub struct EventRepository<T> {
    _conn: T,
}

impl<T> EventRepository<T> {
    pub fn new(conn: T) -> EventRepository<T> {
        EventRepository { _conn: conn }
    }
}

impl<T> common::EventRepository for EventRepository<T> {
    fn save_event(&self) {}
}

#[derive(Clone)]
pub struct SqliteConnectionPool(Arc<Mutex<sqlite::Connection>>);

impl SqliteConnectionPool {
    pub fn new(conn: sqlite::Connection) -> Self {
        Self(Arc::new(Mutex::new(conn)))
    }

    pub fn get<'a>(&'a self) -> SqliteConnection<'a> {
        SqliteConnection::new(self.0.lock().unwrap())
    }
}

#[derive(Clone)]
pub struct SqliteConnection<'a>(Arc<MutexGuard<'a, sqlite::Connection>>);

impl<'a> SqliteConnection<'a> {
    fn new(conn: MutexGuard<'a, sqlite::Connection>) -> SqliteConnection<'a> {
        SqliteConnection(Arc::new(conn))
    }
}

pub struct MigrationStatusRepository {
    pool: SqliteConnectionPool,
}

impl MigrationStatusRepository {
    pub fn new(pool: SqliteConnectionPool) -> Result<MigrationStatusRepository> {
        let query = "CREATE TABLE IF NOT EXISTS migration_status (
            name TEXT,
            status TEXT,
            PRIMARY KEY (name)
        );";

        {
            let conn = pool.get();
            conn.0.execute(query)?;
        }

        Ok(MigrationStatusRepository { pool })
    }
}

impl migrations::StatusRepository for MigrationStatusRepository {
    fn get_status(&self, name: &str) -> Result<Option<migrations::Status>> {
        let query = "SELECT status FROM migration_status WHERE name = :name LIMIT 1";

        let conn = self.pool.get();
        let mut statement = conn.0.prepare(query)?;

        statement.bind((":name", name))?;

        if let Ok(sqlite::State::Row) = statement.next() {
            let value: String = statement.read("status")?;
            return Ok(Some(status_from_persisted(&value)?));
        }

        Ok(None)
    }

    fn save_status(&self, name: &str, status: migrations::Status) -> Result<()> {
        let persisted_status = status_to_persisted(&status);

        let conn = self.pool.get();
        let mut statement = conn.0.prepare(
            "INSERT OR REPLACE INTO
            migration_status(name, status)
            VALUES (:name, :status)
        ",
        )?;
        statement.bind((":name", name))?;
        statement.bind((":status", persisted_status.as_str()))?;
        statement.next()?;

        Ok(())
    }
}

const STATUS_FAILED: &str = "failed";
const STATUS_COMPLETED: &str = "completed";

fn status_to_persisted(status: &migrations::Status) -> String {
    match status {
        migrations::Status::Failed => STATUS_FAILED,
        migrations::Status::Completed => STATUS_COMPLETED,
    }
    .to_string()
}

fn status_from_persisted(status: &String) -> Result<migrations::Status> {
    return match status.as_str() {
        STATUS_FAILED => Ok(migrations::Status::Failed),
        STATUS_COMPLETED => Ok(migrations::Status::Completed),
        _ => Err(format!("unknown status: {status}"))?,
    };
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::migrations::MigrationCallable;
//
//    #[cfg(test)]
//    mod test_migration_status_repository {
//        use super::*;
//        use crate::errors::Result;
//        use crate::migrations::StatusRepository;
//
//        #[test]
//        fn test_get_status_returns_none_if_repository_is_empty() -> Result<()> {
//            let r = create_repository()?;
//
//            let status = r.get_status("some_name")?;
//            assert!(status.is_none());
//
//            Ok(())
//        }
//
//        #[test]
//        fn test_get_status_returns_saved_status() -> Result<()> {
//            let r = create_repository()?;
//
//            let name = "some_name";
//            let status = migrations::Status::Completed;
//
//            r.save_status(name, status)?;
//
//            let returned_status = r.get_status(name)?;
//            assert!(returned_status.is_some());
//            assert!(returned_status.unwrap() == migrations::Status::Completed);
//
//            Ok(())
//        }
//
//        fn create_repository() -> Result<MigrationStatusRepository> {
//            return MigrationStatusRepository::new(*new_sqlite()?);
//        }
//    }
//
//    #[cfg(test)]
//    mod test_transaction_provider {
//        use super::*;
//        use crate::errors::Result;
//        use crate::service::app::common::TransactionProvider as TransactionProviderTrait;
//
//        #[test]
//        fn aaa() -> Result<()> {
//            let provider = new()?;
//            let mut transaction = provider.start_transaction()?;
//            transaction.commit()?;
//            // todo add tests
//            Ok(())
//        }
//
//        fn new() -> Result<TransactionProvider> {
//            let adapter = new_sqlite()?;
//            let provider = TransactionProvider::new(*adapter);
//            Ok(provider)
//        }
//    }
//
//    #[cfg(test)]
//    mod test_registration_repository {
//        use super::*;
//        use crate::fixtures;
//        use common::RegistrationRepository as _;
//
//        #[test]
//        fn test_save_registration() -> Result<()> {
//            let repo = create_repository()?;
//            let registration1 = create_registration()?;
//            let registration2 = create_registration()?;
//
//            repo.save(&registration1)?;
//            repo.save(&registration2)?;
//
//            let mut all_relays = vec![];
//            all_relays.append(&mut registration1.relays());
//            all_relays.append(&mut registration2.relays());
//
//            let mut retrieved_relays = repo.get_relays()?;
//            assert_eq!(retrieved_relays.sort(), all_relays.sort());
//
//            for relay in registration1.relays() {
//                let pub_keys = repo.get_pub_keys(&relay)?;
//                assert_eq!(
//                    pub_keys,
//                    vec![common::PubKeyInfo::new(registration1.pub_key())]
//                );
//            }
//
//            for relay in registration2.relays() {
//                let pub_keys = repo.get_pub_keys(&relay)?;
//                assert_eq!(
//                    pub_keys,
//                    vec![common::PubKeyInfo::new(registration2.pub_key())]
//                );
//            }
//
//            Ok(())
//        }
//
//        fn create_registration() -> Result<domain::Registration> {
//            let pub_key = fixtures::some_pub_key();
//            let apns_token = fixtures::some_apns_token();
//            let locale = fixtures::some_locale();
//            let relays = vec![
//                fixtures::some_relay_address(),
//                fixtures::some_relay_address(),
//            ];
//
//            let registration = domain::Registration::new(pub_key, apns_token, relays, locale)?;
//            Ok(registration)
//        }
//
//        fn create_repository() -> Result<RegistrationRepository<'static>> {
//            Ok(RegistrationRepository::new(*new_conn()?))
//        }
//
//
//    }
//
//    fn new_sqlite() -> Result<Box<SqliteConnectionPool>> {
//        let pool = SqliteConnectionPool::new(sqlite::open(":memory:")?);
//        RegistrationRepositoryMigration0001::new(pool.clone()).run()?;
//        Ok(Box::new(pool))
//    }
//
//    fn new_conn() -> Result<Box<SqliteConnection<'static>>> {
//        let pool = new_sqlite()?;
//        Ok(Box::new(pool.get()))
//    }
//}
