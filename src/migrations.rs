use crate::errors::Result;
use std::collections::HashSet;

pub type MigrationFn = fn() -> Result<()>;

pub struct Migration {
    name: String,
    run_fn: MigrationFn,
}

impl Migration {
    pub fn new(name: String, run_fn: MigrationFn) -> Result<Migration> {
        if name.is_empty() {
            return Err("empty name")?;
        }
        return Ok(Migration { name, run_fn });
    }
}

pub struct Migrations {
    migrations: Vec<Migration>,
}

impl Migrations {
    pub fn new(migrations: Vec<Migration>) -> Result<Migrations> {
        let mut names = HashSet::new();

        for migration in &migrations {
            if names.contains(&migration.name) {
                return Err("duplicate migration name")?;
            }
            names.insert(migration.name.clone());
        }

        return Ok(Migrations { migrations });
    }
}

#[derive(PartialEq)]
pub enum Status {
    Failed,
    Completed,
}

pub trait StatusRepository {
    fn get_status(&self, name: &str) -> Result<Option<Status>>;
    fn save_status(&self, name: &str, status: Status) -> Result<()>;
}

pub struct Runner<T: StatusRepository> {
    status_repository: T,
}

impl<T: StatusRepository> Runner<T> {
    pub fn new(status_repository: T) -> Runner<T> {
        return Runner { status_repository };
    }

    pub fn run(&self, migrations: &Migrations) -> Result<()> {
        for migration in &migrations.migrations {
            let status = self.status_repository.get_status(&migration.name)?;

            match status {
                Some(status) => match status {
                    Status::Completed => continue,
                    Status::Failed => {} // run migration
                },
                None => {} // run migration
            }

            match (migration.run_fn)() {
                Ok(_) => {
                    self.status_repository
                        .save_status(&migration.name, Status::Completed)?;
                    continue;
                }
                Err(err) => {
                    self.status_repository
                        .save_status(&migration.name, Status::Failed)?;
                    return Err(err);
                }
            }
        }

        return Ok(());
    }
}
