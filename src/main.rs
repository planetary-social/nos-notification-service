mod errors;
mod migrations;
mod service;

use crate::errors::Result;
use crate::service::app;
use crate::service::app::commands::implementation as commandsimpl;
use crate::service::app::common;
use crate::service::ports::http;
use service::adapters::sqlite as sqliteadapters;

fn main() {
    let conn = sqlite::Connection::open("/tmp/db.sqlite").unwrap();
    let adapters_factory_fn = new_adapters_factory_fn();
    let migrations = new_migrations(&conn).unwrap();

    let transactionProvider = sqliteadapters::TransactionProvider::new(&conn, adapters_factory_fn);

    let register = commandsimpl::RegisterHandler::new();

    let commands = app::Commands::new(&register);
    let queries = app::Queries::new();
    let app = app::Application::new(&commands, &queries);

    let migration_status_repository = sqliteadapters::MigrationStatusRepository::new(&conn).unwrap();
    let runner = migrations::Runner::new(migration_status_repository);

    let server = http::Server::new(&app);

    runner.run(&migrations).unwrap();
    server.listen_and_serve();
}

fn new_adapters_factory_fn() -> Box<sqliteadapters::AdaptersFactoryFn> {
    return Box::new(constrain(|conn: &sqlite::Connection| -> common::Adapters {
        let registrations = Box::new(sqliteadapters::RegistrationRepository::new(conn));
        return common::Adapters {
            registrations: registrations,
        };
    }));
}

fn new_migrations<'a>(conn: &'a sqlite::Connection) -> Result<migrations::Migrations<'a>> {
    let mut migrations: Vec<migrations::Migration> = Vec::new();

    for migration in sqliteadapters::RegistrationRepository::migrations(&conn)? {
        migrations.push(migration);
    }

    return migrations::Migrations::new(migrations);
}

fn constrain<F>(f: F) -> F
where
    F: for<'a> Fn(&'a sqlite::Connection) -> common::Adapters<'a>,
{
    f
}
