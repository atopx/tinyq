use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// 指令, 固定3个字节长度
#[derive(Debug, Deserialize, Serialize, EnumString, Display, Eq, PartialEq)]
pub enum Instruction {
    #[strum(to_string = "aup")]
    Aup,
    #[strum(to_string = "pub")]
    Pub,
    #[strum(to_string = "sub")]
    Sub,
    #[strum(to_string = "rdy")]
    Rdy,
    #[strum(to_string = "ack")]
    Ack,
    #[strum(to_string = "cls")]
    Cls,
    #[strum(to_string = "del")]
    Del,
}

impl Instruction {
    pub fn from_slice(slice: Vec<u8>) -> Option<Self> {
        match String::from_utf8(slice) {
            Ok(value) => match Self::from_str(&value.to_lowercase()) {
                Ok(ins) => Some(ins),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
}
