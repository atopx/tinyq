use super::ecode;

pub async fn handle(body: Vec<u8>) -> (u8, Vec<u8>) {
    println!("INSTRUCTION ack {}", String::from_utf8(body).unwrap());
    (ecode::SUCCESS, vec![])
}
