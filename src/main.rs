mod service;

use crate::service::ports::http;

fn main() {
    http::listen_and_serve();
}
