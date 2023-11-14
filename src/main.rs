use instruction::{ecode, Instruction};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod instruction;

const MAX_BODY_SIZE: u32 = 1024 * 100 * 100;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("Server running on: {}", addr);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_client(socket));
    }
}

async fn handle_client(socket: TcpStream) {
    let (mut reader, mut writer) = socket.into_split();

    // read instruction
    let instruct: Instruction = if let Ok(v) = reader.read_u8().await {
        v.into()
    } else {
        writer
            .write(&[ecode::INS_PARSE_ERR])
            .await
            .unwrap_or_default();
        return;
    };

    if instruct == instruction::NONE {
        writer
            .write(&[ecode::INS_INVAL_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // read body size
    let body_size: u32 = match reader.read_u32().await {
        Ok(v) => v,
        Err(_) => {
            writer
                .write(&[ecode::BODY_SIZE_PARSE_ERR])
                .await
                .unwrap_or_default();
            return;
        }
    };

    // body_size too large
    if body_size == 0 || body_size > MAX_BODY_SIZE {
        writer
            .write(&[ecode::BODY_SIZE_INVAL_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // read body
    let mut body_buf = vec![0u8; body_size as usize];
    if reader.read(&mut body_buf).await.is_err() {
        writer
            .write(&[ecode::BODY_PARAM_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // handle instruction
    let (code, size, data) = instruction::handle(instruct, body_buf).await;
    writer.write(&[code]).await.unwrap_or_default();
    writer.write(&size.to_be_bytes()).await.unwrap_or_default();
    if size > 0 {
        writer.write_all(&data).await.unwrap_or_default();
    }
}
