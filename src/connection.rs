use std::io;
use std::net::SocketAddr;

use bytes::Bytes;
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::time::Duration;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::command::Command;
use crate::config::MAX_BODY_SIZE;
use crate::ecode::Result;
use crate::ecode::StatusCode;

#[derive(Debug)]
pub struct Connection {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<TcpStream>,
    client: SocketAddr,
}

impl Connection {
    /// Create a new `Connection`, backed by `socket`. Read and write buffers
    /// are initialized.
    pub fn new(socket: TcpStream) -> Connection {
        let client = socket.peer_addr().unwrap();
        // client.ip()
        Connection { client, stream: BufWriter::new(socket) }
    }

    pub async fn auth(&mut self) -> Result<()> {
        if self.write_code(StatusCode::InputPassword).await.is_err() {
            return Err(StatusCode::ServerInternalErr);
        }
        let mut buffer = BytesMut::with_capacity(64);
        match timeout(Duration::from_secs(30), self.stream.read_buf(&mut buffer)).await {
            Ok(future) => {
                if future.is_err() {
                    return Err(StatusCode::AuthErr);
                };
                let s = String::from_utf8_lossy(&buffer);
                debug!("read client[{}] password: {}", self.client, s);
                match String::from_utf8_lossy(&buffer).trim() {
                    crate::config::PASSWORD => {
                        info!("client[{}] auth success", self.client);
                        if self.write_code(StatusCode::Success).await.is_err() {
                            return Err(StatusCode::ServerInternalErr);
                        };
                        Ok(())
                    }
                    _ => Err(StatusCode::AuthErr),
                }
            }
            Err(e) => {
                error!("{}", e.to_string());
                Err(StatusCode::AuthErr)
            }
        }
    }

    pub async fn read_command(&mut self) -> Result<Command> {
        match self.stream.read_u8().await {
            Ok(v) => {
                let body = self.read_body().await?;
                Ok(Command::new(v, body).await?)
            }
            Err(_) => Err(StatusCode::CmdParasErr),
        }
    }

    async fn read_body_size(&mut self) -> Result<u32> {
        match self.stream.read_u32().await {
            Ok(v) => {
                if v > MAX_BODY_SIZE {
                    Err(StatusCode::BodySizeInvalErr)
                } else {
                    Ok(v)
                }
            }
            Err(_) => Err(StatusCode::BodySizeParseErr),
        }
    }

    pub async fn read_body(&mut self) -> Result<Bytes> {
        let body_size = self.read_body_size().await?;
        if body_size == 0 {
            return Ok(Bytes::new());
        }
        let mut buffer = BytesMut::with_capacity(body_size as usize);
        if self.stream.read_buf(&mut buffer).await.is_err() {
            return Err(StatusCode::BodyParseErr);
        }
        Ok(buffer.into())
    }

    pub async fn write_code(&mut self, code: StatusCode) -> io::Result<()> {
        self.stream.write_u8(code.to_byte()).await?;
        self.stream.flush().await
    }

    pub async fn write_data(&mut self, data: Bytes) -> io::Result<()> {
        self.stream.write_u8(StatusCode::Success.to_byte()).await?;
        self.stream.write_u32(data.len() as u32).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await
    }
}
