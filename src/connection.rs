use bytes::{Bytes, BytesMut};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::command::Command;
use crate::config::MAX_BODY_SIZE;
use crate::ecode::{ECode, Result};

#[derive(Debug)]
pub struct Connection {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<TcpStream>,
}

impl Connection {
    /// Create a new `Connection`, backed by `socket`. Read and write buffers
    /// are initialized.
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
        }
    }

    pub async fn read_command(&mut self) -> Result<Command> {
        match self.stream.read_u8().await {
            Ok(v) => match Command::from_u8(v) {
                Some(cmd) => {
                    let body = self.read_body().await?;
                    cmd.parse(body).await?;
                    Ok(cmd)
                }
                None => Err(ECode::CmdInvalErr),
            },
            Err(_) => Err(ECode::CmdParasErr),
        }
    }

    async fn read_body_size(&mut self) -> Result<u32> {
        match self.stream.read_u32().await {
            Ok(v) => {
                if v > MAX_BODY_SIZE {
                    Err(ECode::BodySizeInvalErr)
                } else {
                    Ok(v)
                }
            }
            Err(_) => Err(ECode::BodySizeParseErr),
        }
    }

    pub async fn read_body(&mut self) -> Result<Bytes> {
        let body_size = self.read_body_size().await?;
        let mut buffer = BytesMut::with_capacity(body_size as usize);
        if self.stream.read_buf(&mut buffer).await.is_err() {
            return Err(ECode::BodyParseErr);
        }
        Ok(buffer.into())
    }

    pub async fn write_error(&mut self, code: ECode) -> io::Result<()> {
        self.stream.write_u8(code.to_byte()).await?;
        self.stream.flush().await
    }

    pub async fn write_data(&mut self, data: Bytes) -> io::Result<()> {
        self.stream.write_u8(ECode::Success.to_byte()).await?;
        self.stream.write_u32(data.len() as u32).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await
    }
}
