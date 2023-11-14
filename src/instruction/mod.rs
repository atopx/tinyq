use crate::store::Queues;

pub mod ecode;
pub mod ins_ack;
pub mod ins_auth;
pub mod ins_clean;
pub mod ins_del;
pub mod ins_pub;
pub mod ins_rdy;
pub mod ins_sub;

pub type Instruction = u8;

pub const NONE: Instruction = 0x00;
pub const AUTH: Instruction = 0x01;
pub const PUB: Instruction = 0x02;
pub const SUB: Instruction = 0x03;
pub const RDY: Instruction = 0x04;
pub const ACK: Instruction = 0x05;
pub const CLEAN: Instruction = 0x06;
pub const DEL: Instruction = 0x07;

pub async fn handle(
    queues: Queues,
    ins: Instruction,
    body: Vec<u8>,
) -> (u8, u32, Vec<u8>) {
    let (code, data): (ecode::ECode, Vec<u8>) = match ins {
        AUTH => ins_auth::handle(queues, body).await,
        PUB => ins_pub::handle(queues, body).await,
        SUB => ins_sub::handle(queues, body).await,
        RDY => ins_rdy::handle(queues, body).await,
        ACK => ins_ack::handle(queues, body).await,
        DEL => ins_del::handle(queues, body).await,
        CLEAN => ins_clean::handle(queues, body).await,
        _ => panic!(""),
    };
    (code, data.len() as u32, data)
}
