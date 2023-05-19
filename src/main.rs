mod errors;
mod migrations;
mod service;

#[cfg(test)]
mod fixtures;

use crate::service::app;
use crate::service::app::commands::implementation as commandsimpl;
use crate::service::app::common;
use crate::service::ports::http;
use service::adapters::sqlite::{
    self as sqliteadapters, SqliteConnection, SqliteConnectionAdapter,
};
use std::borrow::Borrow;

fn main() {
    let conn = sqlite::Connection::open("/tmp/db.sqlite").unwrap();
    let conn_adapter = sqliteadapters::SqliteConnectionAdapter(conn);

    //let adapters_factory_fn: Box<
    //    AdaptersFactoryFn<
    //        SqliteConnectionAdapter,
    //        Adapters<sqliteadapters::RegistrationRepository<SqliteConnectionAdapter>>,
    //    >,
    //> = new_adapters_factory_fn();

    let adapters_factory_fn = new_adapters_factory_fn();

    let mut migrations: Vec<migrations::Migration> = Vec::new();

    // no idea how to make a factory function for this without ownership errors
    let migration_registration_0001_create_tables =
        sqliteadapters::RegistrationRepositoryMigration0001::new::<SqliteConnectionAdapter>(
            Borrow::borrow(&conn_adapter),
        );

    migrations.push(
        migrations::Migration::new(
            "registration.0001_create_tables",
            &migration_registration_0001_create_tables,
        )
        .unwrap(),
    );

    let migrations = migrations::Migrations::new(migrations).unwrap();

    let transaction_provider: sqliteadapters::TransactionProvider<
        sqliteadapters::SqliteConnectionAdapter,
        AdaptersImpl<&SqliteConnectionAdapter>,
    > = sqliteadapters::TransactionProvider::new(
        Borrow::borrow(&conn_adapter),
        adapters_factory_fn,
    );

    let register = commandsimpl::RegisterHandler::new();

    let commands = app::Commands::new(&register);
    let queries = app::Queries::new();
    let app = app::Application::new(&commands, &queries);

    let migration_status_repository = sqliteadapters::MigrationStatusRepository::new::<
        SqliteConnectionAdapter,
    >(Borrow::borrow(&conn_adapter))
    .unwrap();
    let runner = migrations::Runner::new(migration_status_repository);

    let server = http::Server::new(&app);

    runner.run(&migrations).unwrap();
    server.listen_and_serve();
}

type AdaptersImpl<T> = common::Adapters<sqliteadapters::RegistrationRepository<T>>;

fn new_adapters_factory_fn<T>() -> Box<sqliteadapters::AdaptersFactoryFn<T, AdaptersImpl<T>>>
where
    T: SqliteConnection,
{
    return Box::new(
        |conn: T| -> common::Adapters<sqliteadapters::RegistrationRepository<T>> {
            let registrations = sqliteadapters::RegistrationRepository::new(conn);
            return common::Adapters {
                registrations: registrations,
            };
        },
    );
}

fn constrain<F, C, A>(f: F) -> F
where
    F: for<'a> Fn(C) -> A,
{
    f
}
