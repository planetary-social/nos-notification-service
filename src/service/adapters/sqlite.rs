use crate::errors::Result;
use crate::migrations;
use crate::service::app::common;
use crate::service::domain;
use sqlite;
use sqlite::State;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone)]
pub struct TransactionProvider {
    conn: SqliteConnectionAdapter,
}

impl TransactionProvider {
    pub fn new(conn: SqliteConnectionAdapter) -> TransactionProvider {
        TransactionProvider { conn }
    }

    fn new_adapters(&self) -> common::Adapters {
        let registrations = Box::new(RegistrationRepository::new(self.conn.clone()));
        let events = Box::new(EventRepository::new(self.conn.clone()));
        common::Adapters::new(registrations, events)
    }
}

impl common::TransactionProvider for TransactionProvider {
    fn start_transaction(&self) -> Result<Box<dyn common::Transaction>> {
        let conn = self.conn.clone();

        conn.get().execute("BEGIN TRANSACTION")?;

        let adapters = self.new_adapters();
        let t = Transaction::new(self.conn.clone(), adapters);
        Ok(Box::new(t))
    }
}

struct Transaction {
    _conn: SqliteConnectionAdapter,
    adapters: common::Adapters,
    _commited: bool,
}

impl Transaction {
    fn new(conn: SqliteConnectionAdapter, adapters: common::Adapters) -> Self {
        Self {
            _conn: conn,
            adapters,
            _commited: false,
        }
    }
}

impl common::Transaction for Transaction {
    fn adapters(&self) -> common::Adapters {
        self.adapters.clone()
    }

    fn commit(&self) -> Result<()> {
        //match f(adapters) {
        //    Ok(r) => {
        //        self.conn.execute("COMMIT TRANSACTION")?;
        //        Ok(r)
        //    }
        //    Err(err) => {
        //        self.conn.execute("ROLLBACK TRANSACTION")?;
        //        Err(err)
        //    }
        //}
        Ok(())
    }
}

pub struct RegistrationRepository {
    conn: SqliteConnectionAdapter,
}

impl RegistrationRepository {
    pub fn new(conn: SqliteConnectionAdapter) -> RegistrationRepository {
        RegistrationRepository { conn }
    }
}

impl common::RegistrationRepository for RegistrationRepository {
    fn save(&self, registration: &domain::Registration) -> Result<()> {
        let hex_public_key = registration.pub_key().hex();

        let conn = self.conn.get();

        let mut statement = conn.prepare(
            "INSERT OR REPLACE INTO
            registration(public_key, apns_token, locale)
            VALUES (:public_key, :apns_token, :locale)
        ",
        )?;
        statement.bind((":public_key", hex_public_key.as_str()))?;
        statement.bind((":apns_token", registration.apns_token().as_ref()))?;
        statement.bind((":locale", registration.locale().as_ref()))?;
        statement.next()?;

        let mut statement = conn.prepare("DELETE FROM relays WHERE public_key=:public_key")?;
        statement.bind((":public_key", hex_public_key.as_str()))?;
        statement.next()?;

        for address in registration.relays() {
            let mut statement = conn.prepare(
                "INSERT INTO relays (public_key, address) VALUES (:public_key, :address)",
            )?;
            statement.bind((":public_key", hex_public_key.as_str()))?;
            statement.bind((":address", address.as_ref()))?;
            statement.next()?;
        }

        Ok(())
    }

    fn get_relays(&self) -> Result<Vec<domain::RelayAddress>> {
        let conn = self.conn.get();
        let query = "SELECT address FROM relays GROUP BY address";
        let mut statement = conn.prepare(query)?;

        let mut relay_addresses = Vec::new();

        while let Ok(State::Row) = statement.next() {
            let address_string = statement.read::<String, _>("address")?;
            let relay_address = domain::RelayAddress::new(address_string)?;
            relay_addresses.push(relay_address);
        }

        Ok(relay_addresses)
    }

    fn get_pub_keys(&self, address: &domain::RelayAddress) -> Result<Vec<common::PubKeyInfo>> {
        let conn = self.conn.get();
        let query = "SELECT public_key FROM relays WHERE address = :address";
        let mut statement = conn.prepare(query)?;
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
    conn: SqliteConnectionAdapter,
}

impl RegistrationRepositoryMigration0001 {
    pub fn new(conn: SqliteConnectionAdapter) -> RegistrationRepositoryMigration0001 {
        RegistrationRepositoryMigration0001 { conn }
    }
}

impl migrations::MigrationCallable for RegistrationRepositoryMigration0001 {
    fn run(&self) -> Result<()> {
        let conn = self.conn.get();

        conn.execute(
            "CREATE TABLE registration (
              public_key TEXT,
              apns_token TEXT,
              locale TEXT,
              PRIMARY KEY (public_key)
             )",
        )?;

        conn.execute(
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
pub struct SqliteConnectionAdapter(Arc<Mutex<sqlite::Connection>>);

impl SqliteConnectionAdapter {
    pub fn new(conn: sqlite::Connection) -> Self {
        Self(Arc::new(Mutex::new(conn)))
    }

    pub fn get(&self) -> MutexGuard<sqlite::Connection> {
        self.0.lock().unwrap()
    }
}

pub struct MigrationStatusRepository {
    conn: SqliteConnectionAdapter,
}

impl MigrationStatusRepository {
    pub fn new(conn: SqliteConnectionAdapter) -> Result<MigrationStatusRepository> {
        let query = "CREATE TABLE IF NOT EXISTS migration_status (
            name TEXT,
            status TEXT,
            PRIMARY KEY (name)
        );";

        conn.get().execute(query)?;

        Ok(MigrationStatusRepository { conn })
    }
}

impl migrations::StatusRepository for MigrationStatusRepository {
    fn get_status(&self, name: &str) -> Result<Option<migrations::Status>> {
        let query = "SELECT status FROM migration_status WHERE name = :name LIMIT 1";

        let conn = self.conn.get();
        let mut statement = conn.prepare(query)?;

        statement.bind((":name", name))?;

        if let Ok(sqlite::State::Row) = statement.next() {
            let value: String = statement.read("status")?;
            return Ok(Some(status_from_persisted(&value)?));
        }

        Ok(None)
    }

    fn save_status(&self, name: &str, status: migrations::Status) -> Result<()> {
        let persisted_status = status_to_persisted(&status);

        let conn = self.conn.get();
        let mut statement = conn.prepare(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::MigrationCallable;

    #[cfg(test)]
    mod test_migration_status_repository {
        use super::*;
        use crate::errors::Result;
        use crate::migrations::StatusRepository;

        #[test]
        fn test_get_status_returns_none_if_repository_is_empty() -> Result<()> {
            let r = create_repository()?;

            let status = r.get_status("some_name")?;
            assert!(status.is_none());

            Ok(())
        }

        #[test]
        fn test_get_status_returns_saved_status() -> Result<()> {
            let r = create_repository()?;

            let name = "some_name";
            let status = migrations::Status::Completed;

            r.save_status(name, status)?;

            let returned_status = r.get_status(name)?;
            assert!(returned_status.is_some());
            assert!(returned_status.unwrap() == migrations::Status::Completed);

            Ok(())
        }

        fn create_repository() -> Result<MigrationStatusRepository> {
            return MigrationStatusRepository::new(new_sqlite()?);
        }
    }

    #[cfg(test)]
    mod test_transaction_provider {
        use super::*;
        use crate::errors::Result;
        use crate::service::app::common::TransactionProvider as TransactionProviderTrait;

        #[test]
        fn aaa() -> Result<()> {
            let provider = new()?;
            let transaction = provider.start_transaction()?;
            transaction.commit()?;
            // todo add tests
            Ok(())
        }

        fn new() -> Result<TransactionProvider> {
            let adapter = new_sqlite()?;
            let provider = TransactionProvider::new(adapter);
            Ok(provider)
        }
    }

    #[cfg(test)]
    mod test_registration_repository {
        use super::*;
        use crate::fixtures;
        use common::RegistrationRepository as _;

        #[test]
        fn test_save_registration() -> Result<()> {
            let repo = create_repository()?;
            let registration1 = create_registration()?;
            let registration2 = create_registration()?;

            repo.save(&registration1)?;
            repo.save(&registration2)?;

            let mut all_relays = vec![];
            all_relays.append(&mut registration1.relays());
            all_relays.append(&mut registration2.relays());

            let mut retrieved_relays = repo.get_relays()?;
            assert_eq!(retrieved_relays.sort(), all_relays.sort());

            for relay in registration1.relays() {
                let pub_keys = repo.get_pub_keys(&relay)?;
                assert_eq!(
                    pub_keys,
                    vec![common::PubKeyInfo::new(registration1.pub_key())]
                );
            }

            for relay in registration2.relays() {
                let pub_keys = repo.get_pub_keys(&relay)?;
                assert_eq!(
                    pub_keys,
                    vec![common::PubKeyInfo::new(registration2.pub_key())]
                );
            }

            Ok(())
        }

        fn create_registration() -> Result<domain::Registration> {
            let pub_key = fixtures::some_pub_key();
            let apns_token = fixtures::some_apns_token();
            let locale = fixtures::some_locale();
            let relays = vec![
                fixtures::some_relay_address(),
                fixtures::some_relay_address(),
            ];

            let registration = domain::Registration::new(pub_key, apns_token, relays, locale)?;
            Ok(registration)
        }

        fn create_repository() -> Result<RegistrationRepository> {
            Ok(RegistrationRepository::new(new_sqlite()?))
        }
    }

    fn new_sqlite() -> Result<SqliteConnectionAdapter> {
        let conn = SqliteConnectionAdapter::new(sqlite::open(":memory:")?);
        RegistrationRepositoryMigration0001::new(conn.clone()).run()?;
        Ok(conn)
    }
}
