use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;

use instruction::{ecode, Instruction};

mod config;
mod connection;
mod instruction;
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
