mod service;

use crate::service::app;
use crate::service::app::commands::implementation as commandsimpl;
use crate::service::ports::http;

fn main() {
    let register = commandsimpl::RegisterHandler::new();

    let commands = app::Commands::new(&register);
    let queries = app::Queries::new();
    let app = app::Application::new(&commands, &queries);

    let server = http::Server::new(&app);
    server.listen_and_serve();
}
