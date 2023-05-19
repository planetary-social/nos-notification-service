#![forbid(unsafe_code)]
mod errors;
mod migrations;
mod service;

#[cfg(test)]
mod fixtures;

use crate::service::app;
use crate::service::app::commands::implementation as commandsimpl;
use crate::service::ports::http;
use service::adapters::sqlite as sqliteadapters;
use service::app::commands::downloader::Downloader;

fn main() {
    let conn = sqlite::Connection::open("/tmp/db.sqlite").unwrap();
    let conn_adapter = sqliteadapters::SqliteConnectionAdapter::new(conn);

    //let adapters_factory_fn = new_adapters_factory_fn();

    let mut migrations: Vec<migrations::Migration> = Vec::new();

    // no idea how to make a factory function for this without ownership errors
    let migration_registration_0001_create_tables =
        sqliteadapters::RegistrationRepositoryMigration0001::new(conn_adapter.clone());

    migrations.push(
        migrations::Migration::new(
            "registration.0001_create_tables",
            &migration_registration_0001_create_tables,
        )
        .unwrap(),
    );

    let migrations = migrations::Migrations::new(migrations).unwrap();

    let transaction_provider = sqliteadapters::TransactionProvider::new(conn_adapter.clone());

    let register = commandsimpl::RegisterHandler::new();

    let commands = app::Commands::new(&register);
    let queries = app::Queries::new();
    let app = app::Application::new(&commands, &queries);

    let migration_status_repository =
        sqliteadapters::MigrationStatusRepository::new(conn_adapter).unwrap();
    let runner = migrations::Runner::new(migration_status_repository);

    let server = http::Server::new(&app);

    let _downloader = Downloader::new(transaction_provider);

    runner.run(&migrations).unwrap();
    server.listen_and_serve();
}

//fn new_adapters_factory_fn<T>() -> Box<dyn sqliteadapters::AdaptersFactoryFn<T, AdaptersImpl<T>>>
//where
//    T: SqliteConnection,
//{
//    Box::new(|conn: T| -> AdaptersImpl<T> {
//        let registrations = sqliteadapters::RegistrationRepository::new(conn);
//        let events = sqliteadapters::EventRepository::new(conn);
//        common::Adapters {
//            registrations,
//            events,
//        }
//    })
//}
