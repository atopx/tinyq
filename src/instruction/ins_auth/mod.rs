use crate::store::Queues;

use super::ecode;

#[derive(Debug)]
pub struct Params {}

pub async fn handle(queue: Queues, body: Vec<u8>) -> (u8, Vec<u8>) {
    println!("INSTRUCTION auth {}", String::from_utf8(body).unwrap());

    // 1. 校验身份信息
    // 2. 调整一些配置参数

    (ecode::SUCCESS, vec![])
}
