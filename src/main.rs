use std::net::TcpListener;
use zero2prod::startup::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // TODO: accept port cli option
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to port");
    run(listener)?.await
}
