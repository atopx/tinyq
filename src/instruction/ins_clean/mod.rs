use super::ecode;
use crate::store::Queues;

pub async fn handle(queue: Queues, body: Vec<u8>) -> (u8, Vec<u8>) {
    println!("INSTRUCTION clean {}", String::from_utf8(body).unwrap());
    (ecode::SUCCESS, vec![])
}
