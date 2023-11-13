use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// Define the IdentifyBody struct
#[derive(Debug, Deserialize, Serialize)]
struct AuthBody {
    client_id: Option<String>,
    hostname: String,
    user_agent: String,
    feature_negotiation: bool,
    tls_v1: bool,
    deflate: bool,
    snappy: bool,
    sample_rate: Option<u8>,
}

const INSTRUCTION_AUTH: &str = "AUTH";
const INSTRUCTION_PUB: &str = "PUB";
const INSTRUCTION_SUB: &str = "SUB";
// const INSTRUCTION_SUB: &str = "SUB";
const MAX_BODY_SIZE: u32 = 10_2400;

// Define the Message enum to represent different message types
#[derive(Debug, Deserialize, Serialize)]
enum Message {
    Auth(AuthBody),
    // 认证
    Pub(String, Vec<u8>),
    // 发布
    Sub(String, String),
    // 订阅
    Rdy(u32),
    // 订阅者准备好接收(1-n)条消息
    Ack(String),
    // 消费成功
    Clean(String),
    // 清空队列/主题
    Del(String), // 删除队列/主题
}

#[derive(Debug, Deserialize, Serialize)]
enum Mode {
    Topic(String),
    Queue(String),
}

// TODO: Implement your message processing logic here

// Function to handle IDENTIFY messages
fn handle_identify(identify_body: AuthBody) -> String {
    // TODO: Implement your IDENTIFY message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

// Function to handle SUB messages
fn handle_sub(namespace: String, channel_name: String) -> String {
    // TODO: Implement your SUB message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

// Function to handle PUB messages
fn handle_pub(namespace: String, data: Vec<u8>) -> String {
    // TODO: Implement your PUB message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

// Function to handle RDY messages
fn handle_rdy(count: u32) -> String {
    // TODO: Implement your RDY message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

// Function to handle FIN messages
fn handle_ack(message_id: String) -> String {
    // TODO: Implement your FIN message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

fn handle_clean(namespace: String) -> String {
    // TODO: Implement your FIN message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

fn handle_del(namespace: String) -> String {
    // TODO: Implement your FIN message processing logic
    // For now, just return a placeholder response
    "OK".to_string()
}

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
    let mut instruction: Vec<u8> = Vec::new();
    while let Ok(v) = reader.read_u8().await {
        if v == b' ' {
            break;
        }
        instruction.push(v);
    }
    let instruction = String::from_utf8(instruction).unwrap();
    let mut err = None;
    // AUTH body_size(u32) body([u8])
    match instruction.as_str() {
        "AUTH" => {
            println!("receive instruction: AUTH");
            err = match reader.read_u32().await {
                Ok(size) => {
                    println!("body size is {size}");
                    let mut buffer = vec![0u8; size as usize];
                    println!("{}", String::from_utf8(buffer).unwrap());
                    None
                    // match reader.read(&mut buffer).await {
                    //     Ok(n) => {
                    //         match serde_json::from_slice::<AuthBody>(&buffer) {
                    //             Ok(body) => {
                    //                 println!("auth body[{n}] {body:?}");
                    //                 None
                    //             }
                    //             Err(err) => {
                    //                 println!("{}", String::from_utf8(buffer));
                    //                 Some(format!("deserialize body error: {err}"
                    //             )),
                    //         }
                    // }
                    // Err(err) => {
                    //     Some(format!("read body data error: {err}"))
                    // }
                    // }
                }
                Err(err) => Some(format!("read body size error: {err}")),
            };
        }
        unknown => {
            eprintln!("unknown instruction: {unknown}")
        }
    }

    if let Some(err) = err {
        // response error
        println!("{err}")
    }
}
