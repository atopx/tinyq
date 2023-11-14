use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    tokio::spawn(read_messages(reader));
    let mut message = Vec::new();
    let body = r#"{"client_id": null,"hostname": "test","user_agent": "test","feature_negotiation": true,"tls_v1": true,"deflate": true,"snappy": true,"sample_rate": 1}"#;
    let body = body.as_bytes();
    message.push(0x01);
    message.extend_from_slice(&(body.len() as u32).to_be_bytes());
    message.extend_from_slice(body);
    send_message(&mut writer, &message).await;
    // Wait for user input to terminate the program
    let mut stdin = BufReader::new(io::stdin());
    let mut input = String::new();
    stdin.read_line(&mut input).await.unwrap();
}

async fn send_message<W>(writer: &mut W, message: &[u8])
where
    W: AsyncWriteExt + Unpin,
{
    if let Err(e) = writer.write_all(message).await {
        eprintln!("Error sending message: {:?}", e);
    }
    _ = writer.flush().await;
}

async fn read_messages<R>(mut reader: R)
where
    R: AsyncBufReadExt + Unpin,
{
    let mut buffer = String::new();
    loop {
        if let Ok(n) = reader.read_line(&mut buffer).await {
            if n == 0 {
                // Connection closed
                break;
            }
            println!("Received message from server: {}", buffer.trim());
            buffer.clear();
        } else {
            break;
        }
    }
}
