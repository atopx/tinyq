use tokio::{net::TcpListener, signal};
use tracing::info;

mod command;
mod config;
mod connection;
mod ecode;
mod server;
mod shutdown;
mod store;

#[tokio::main]
async fn main() -> ecode::Result<()> {
    tracing_subscriber::fmt()
        // enable everything
        .with_max_level(tracing::Level::DEBUG)
        // sets this to be the default, global collector for this application.
        .init();

    let addr = "127.0.0.1:25131";
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Server running on: {}", addr);

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
