use crate::store::Queues;

pub mod auth;
pub mod clear;
pub mod del;
pub mod ecode;
pub mod r#pub;
pub mod rdy;
pub mod sub;

pub type Command = u8;

pub const NONE: Command = 0x00;
pub const AUTH: Command = 0x01;
pub const PUB: Command = 0x02;
pub const SUB: Command = 0x03;
pub const RDY: Command = 0x04;
pub const CLEAN: Command = 0x06;
pub const DEL: Command = 0x07;

pub async fn handle(
    queues: Queues,
    ins: Command,
    body: Vec<u8>,
) -> (u8, u32, Vec<u8>) {
    let (code, data): (ecode::ECode, Vec<u8>) = match ins {
        AUTH => auth::handle(queues, body).await,
        PUB => r#pub::handle(queues, body).await,
        SUB => sub::handle(queues, body).await,
        RDY => rdy::handle(queues, body).await,
        DEL => del::handle(queues, body).await,
        CLEAN => clear::handle(queues, body).await,
        _ => (ecode::INS_PARSE_ERR, Vec::new()),
    };
    (code, data.len() as u32, data)
}
