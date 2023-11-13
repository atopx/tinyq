use instruction::Instruction;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod instruction;

const MAX_BODY_SIZE: u32 = 10_2400;

mod ecode;

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
    let mut ins_buf = vec![0; 3];
    if reader.read(&mut ins_buf).await.is_err() {
        writer
            .write_all(&ecode::INS_PARSE_ERR)
            .await
            .unwrap_or_default();
        return;
    }

    let instruct = Instruction::from_slice(ins_buf);

    if instruct.is_none() {
        writer
            .write_all(&ecode::INS_INVAL_ERR)
            .await
            .unwrap_or_default();
        return;
    }

    if reader.read_u8().await.unwrap_or_default() != b' ' {
        writer
            .write_all(&ecode::INS_PARAM_ERR)
            .await
            .unwrap_or_default();
        return;
    }

    // read body size
    let body_size: u32 = match reader.read_u32().await {
        Ok(v) => v,
        Err(_) => {
            writer
                .write_all(&ecode::BODY_SIZE_PARSE_ERR)
                .await
                .unwrap_or_default();
            return;
        }
    };

    if body_size == 0 || body_size > MAX_BODY_SIZE {
        writer
            .write_all(&ecode::BODY_SIZE_INVAL_ERR)
            .await
            .unwrap_or_default();
        return;
    }

    if reader.read_u8().await.unwrap_or_default() != b' ' {
        writer
            .write_all(&ecode::BODY_PARSE_ERROR)
            .await
            .unwrap_or_default();
        return;
    }

    // read body
    let mut body_buf = vec![0u8; body_size as usize];
    if reader.read(&mut body_buf).await.is_err() {
        writer
            .write_all(&ecode::BODY_PARAM_ERR)
            .await
            .unwrap_or_default();
        return;
    }

    // handle instruction
    let (data, code) = handle_instraction(instruct.unwrap(), body_buf).await;

    writer.write_all(&code).await.unwrap_or_default();
    if let Some(data) = data {
        writer.write(&[0x20]).await.unwrap_or_default();
        writer.write_all(&data).await.unwrap_or_default();
    }
}

async fn handle_instraction(
    ins: Instruction,
    body: Vec<u8>,
) -> (Option<Vec<u8>>, [u8; 4]) {
    match ins {
        Instruction::Aup => todo!(),
        Instruction::Pub => todo!(),
        Instruction::Sub => todo!(),
        Instruction::Rdy => todo!(),
        Instruction::Ack => todo!(),
        Instruction::Cls => todo!(),
        Instruction::Del => todo!(),
    }

    (Some(vec![]), ecode::SUCCESS)
}
