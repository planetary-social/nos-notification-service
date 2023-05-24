use crate::errors::Result;
use std::collections::HashSet;

// I give up, no idea how to make this work with closures so that I can return a closure that
// captured something from functions and pass it to the migrations runner.
pub trait MigrationCallable {
    fn run(&self) -> Result<()>;
}

impl<W: MigrationCallable + ?Sized> MigrationCallable for Box<W> {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

pub struct Migration<'a> {
    name: &'a str,
    callable: &'a dyn MigrationCallable,
}

impl Migration<'_> {
    pub fn new<'a>(name: &'a str, callable: &'a dyn MigrationCallable) -> Result<Migration<'a>> {
        if name.is_empty() {
            return Err("empty name")?;
        }

        Ok(Migration { name, callable })
    }
}

pub struct Migrations<'a> {
    migrations: Vec<Migration<'a>>,
}

impl Migrations<'_> {
    pub fn new(migrations: Vec<Migration>) -> Result<Migrations> {
        let mut names = HashSet::new();

        for migration in &migrations {
            if names.contains(&migration.name) {
                return Err("duplicate migration name")?;
            }
            names.insert(migration.name);
        }

        Ok(Migrations { migrations })
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
        Runner { status_repository }
    }

    pub fn run(&self, migrations: &Migrations) -> Result<()> {
        for migration in &migrations.migrations {
            let status = self.status_repository.get_status(migration.name)?;

            if let Some(status) = status {
                match status {
                    Status::Completed => continue,
                    Status::Failed => {} // run migration
                }
            }

            match migration.callable.run() {
                Ok(_) => {
                    self.status_repository
                        .save_status(migration.name, Status::Completed)?;
                    continue;
                }
                Err(err) => {
                    self.status_repository
                        .save_status(migration.name, Status::Failed)?;
                    return Err(From::from(format!(
                        "error running migration '{}': {}",
                        migration.name, err
                    )));
                }
            }
        }

        Ok(())
    }
}
