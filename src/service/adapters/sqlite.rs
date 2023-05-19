use crate::errors::Result;
use crate::migrations;
use crate::service::app::common;
use crate::service::domain;
use sqlite;
use std::borrow::Borrow;

pub type AdaptersFactoryFn<T, A> = dyn Fn(T) -> A;

pub struct TransactionProvider<'a, T, A> {
    //conn: Mutex<T>,
    conn: &'a T,
    factory_fn: Box<AdaptersFactoryFn<&'a T, A>>,
}

impl<'a, T, A> TransactionProvider<'_, T, A>
where
    T: SqliteConnection,
{
    pub fn new<TA>(
        conn: &'a T,
        factory_fn: Box<AdaptersFactoryFn<&'a T, A>>,
    ) -> TransactionProvider<'a, T, A>
    where
        T: Borrow<TA>,
        TA: SqliteConnection,
    {
        return TransactionProvider{
            //conn: Mutex::new(conn),
            conn,
            factory_fn,
        };
    }
}

impl<'a, T, A> common::TransactionProvider<A> for TransactionProvider<'a, T, A>
where
    T: SqliteConnection,
{
    fn transact(&self, f: &common::TransactionFn<A>) -> Result<()> {
        //let conn = self.conn.lock().unwrap();

        self.conn.execute("BEGIN TRANSACTION")?;

        let adapters = (self.factory_fn)(self.conn);
        match f(adapters) {
            Ok(()) => {
                self.conn.execute("COMMIT TRANSACTION")?;
                return Ok(());
            }
            Err(err) => {
                self.conn.execute("ROLLBACK TRANSACTION")?;
                return Err(err);
            }
        }
    }
}

pub struct RegistrationRepository<T> {
    conn: T,
}

impl<T> RegistrationRepository<T>
where
    T: SqliteConnection,
{
    pub fn new<TA>(conn: T) -> RegistrationRepository<T>
    where
        T: Borrow<TA>,
        TA: SqliteConnection,
    {
        return RegistrationRepository { conn };
    }
}

impl<T> common::RegistrationRepository for RegistrationRepository<T>
where
    T: SqliteConnection,
{
    fn save(&self, registration: domain::Registration) -> Result<()> {
        let s = self.conn.prepare("INSERT INTO registration ()");
        return Err("not implemented")?;
    }
}

pub struct RegistrationRepositoryMigration0001<T> {
    conn: T,
}

impl<T> RegistrationRepositoryMigration0001<T>
where
    T: SqliteConnection,
{
    pub fn new<TA>(conn: T) -> RegistrationRepositoryMigration0001<T>
    where
        T: Borrow<TA>,
        TA: SqliteConnection,
    {
        return RegistrationRepositoryMigration0001 { conn };
    }
}

impl<T> migrations::MigrationCallable for RegistrationRepositoryMigration0001<T>
where
    T: SqliteConnection,
{
    fn run(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE registration (
              public_key TEXT,
              apns_token TEXT,
              locale TEXT,
              PRIMARY KEY (public_key)
             )",
        )?;
        return Ok(());
    }
}

pub trait SqliteConnection {
    //fn execute(&self, statement: &str) -> sqlite::Result<()>;
    fn execute<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<()>;
    //fn prepare(&self, statement: &str) -> sqlite::Result<sqlite::Statement<'_>>;
    fn prepare<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<sqlite::Statement<'_>>;
}

pub struct SqliteConnectionAdapter(pub sqlite::Connection);

impl SqliteConnection for SqliteConnectionAdapter {
    fn execute<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<()> {
        return self.0.execute(statement);
    }

    fn prepare<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<sqlite::Statement<'_>> {
        return self.0.prepare(statement);
    }
}

impl SqliteConnection for &SqliteConnectionAdapter {
    fn execute<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<()> {
        return self.0.execute(statement);
    }

    fn prepare<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<sqlite::Statement<'_>> {
        return self.0.prepare(statement);
    }
}

pub struct MigrationStatusRepository<T> {
    conn: T,
}

impl<T> MigrationStatusRepository<T>
where
    T: SqliteConnection,
{
    pub fn new<TA>(conn: T) -> Result<MigrationStatusRepository<T>>
    where
        T: Borrow<TA>,
        TA: SqliteConnection,
    {
        let query = "CREATE TABLE IF NOT EXISTS migration_status (
            name TEXT,
            status TEXT,
            PRIMARY KEY (name)
        );";
        conn.execute(query)?;

        return Ok(MigrationStatusRepository { conn });
    }
}

impl<T> migrations::StatusRepository for MigrationStatusRepository<T>
where
    T: SqliteConnection,
{
    fn get_status(&self, name: &str) -> Result<Option<migrations::Status>> {
        let query = "SELECT status FROM migration_status WHERE name = :name LIMIT 1";

        let mut statement = self.conn.prepare(query)?;
        statement.bind((":name", name))?;

        while let Ok(sqlite::State::Row) = statement.next() {
            let value: String = statement.read("status")?;
            return Ok(Some(status_from_persisted(value)?));
        }

        return Ok(Option::None);
    }

    fn save_status(&self, name: &str, status: migrations::Status) -> Result<()> {
        let query = "INSERT OR REPLACE INTO
        migration_status(name, status)
        VALUES (:name, :status)
        ";

        let persisted_status = status_to_persisted(status);

        let mut statement = self.conn.prepare(query)?;
        statement.bind((":name", name))?;
        statement.bind((":status", persisted_status.as_str()))?;
        statement.next()?;

        return Ok(());
    }
}

const STATUS_FAILED: &str = "failed";
const STATUS_COMPLETED: &str = "completed";

fn status_to_persisted(status: migrations::Status) -> String {
    return match status {
        migrations::Status::Failed => STATUS_FAILED,
        migrations::Status::Completed => STATUS_COMPLETED,
    }
    .to_string();
}

fn status_from_persisted(status: String) -> Result<migrations::Status> {
    return match status.as_str() {
        STATUS_FAILED => Ok(migrations::Status::Failed),
        STATUS_COMPLETED => Ok(migrations::Status::Completed),
        _ => Err(format!("unknown status: {}", status))?,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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

            return Ok(());
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

            return Ok(());
        }

        fn create_repository<'a>() -> Result<MigrationStatusRepository<SqliteConnectionAdapter>> {
            return MigrationStatusRepository::new(new_sqlite()?);
        }
    }

    #[cfg(test)]
    mod test_registration_repository {
        use crate::fixtures;
        use super::*;
        use common::RegistrationRepository as _;

        #[test]
        fn test_save_registration() -> Result<()> {
            let r = create_repository()?;

            let (_sk, pk) = nostr::secp256k1::generate_keypair(&mut rand::rngs::OsRng {});

            let pub_key = domain::PubKey::new(nostr::key::XOnlyPublicKey::from(pk))?;
            let apns_token = domain::APNSToken::new(String::from("apns_token"))?;
            let mut relays = Vec::new();
            relays.push(fixtures::some_relay_address());
            let locale = domain::Locale::new(String::from("some locale"))?;

            let registration = domain::Registration::new(pub_key, apns_token, relays, locale)?;

            r.save(registration)?;

            return Ok(());
        }

        fn create_repository<'a>() -> Result<RegistrationRepository<SqliteConnectionAdapter>> {
            return Ok(RegistrationRepository::new(new_sqlite()?));
        }
    }

    fn new_sqlite() -> Result<SqliteConnectionAdapter> {
        let conn = sqlite::open(":memory:")?;
        return Ok(SqliteConnectionAdapter(conn));
    }
}
