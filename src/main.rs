mod errors;
mod migrations;
mod service;

use crate::service::app;
use crate::service::app::commands::implementation as commandsimpl;
use crate::service::app::common;
use crate::service::ports::http;
use service::adapters::sqlite as sqliteadapters;
use service::app::common::RegistrationRepository;

fn main() {
    let conn = sqlite::Connection::open("/tmp/db.sqlite").unwrap();
    let adapters_factory_fn = new_adapters_factory_fn();

    let transactionProvider = sqliteadapters::TransactionProvider::new(conn, adapters_factory_fn);

    let register = commandsimpl::RegisterHandler::new();

    let commands = app::Commands::new(&register);
    let queries = app::Queries::new();
    let app = app::Application::new(&commands, &queries);

    let server = http::Server::new(&app);
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

fn constrain<F>(f: F) -> F
where
    F: for<'a> Fn(&'a sqlite::Connection) -> common::Adapters<'a>,
{
    f
}
