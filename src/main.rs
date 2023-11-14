use tokio::net::TcpListener;
use tokio::signal;

mod command;
mod config;
mod connection;
mod result;
mod server;
mod store;

#[tokio::main]
async fn main() -> result::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Server running on: {}", addr);

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
