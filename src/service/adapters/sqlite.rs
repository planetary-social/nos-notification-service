use crate::errors::Result;
use crate::migrations;
use crate::service::app::common;
use crate::service::domain;
use sqlite;
use std::sync::Mutex;

pub type AdaptersFactoryFn = fn(&sqlite::Connection) -> common::Adapters;

pub struct TransactionProvider<'a> {
    conn: Mutex<&'a sqlite::Connection>,
    factory_fn: Box<AdaptersFactoryFn>,
}

impl<'a> TransactionProvider<'a> {
    pub fn new(
        conn: &'a sqlite::Connection,
        factory_fn: Box<AdaptersFactoryFn>,
    ) -> TransactionProvider {
        return TransactionProvider {
            conn: Mutex::new(conn),
            factory_fn,
        };
    }
}

impl common::TransactionProvider for TransactionProvider<'_> {
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

    pub fn migrations(conn: &sqlite::Connection) -> Result<Vec<migrations::Migration>> {
        let mut migrations  = Vec::new();
        migrations.push(migrations::Migration::new("registration.0001_create_tables", | | -> Result<()> {
            return Err("not implemented")?;
        })?);
        return Ok(migrations);
    }
}

impl common::RegistrationRepository for RegistrationRepository<'_> {
    fn save(&self, registration: domain::Registration) -> Result<()> {

        return Err("not implemented")?;
    }
}

pub struct MigrationStatusRepository<'a> {
    conn: &'a sqlite::Connection,
}

impl MigrationStatusRepository<'_> {
    pub fn new(conn: &sqlite::Connection) -> Result<MigrationStatusRepository> {
        let query = "CREATE TABLE IF NOT EXISTS migration_status (
            name TEXT,
            status TEXT,
            PRIMARY KEY (name)
        );";
        conn.execute(query)?;

        return Ok(MigrationStatusRepository { conn });
    }
}

impl<'a> migrations::StatusRepository for MigrationStatusRepository<'_> {
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

        fn create_repository<'a>() -> Result<MigrationStatusRepository> {
            let conn = sqlite::open(":memory:")?;
            return MigrationStatusRepository::new(conn);
        }
    }
}
