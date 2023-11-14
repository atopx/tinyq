pub const MAX_BODY_SIZE: u32 = 1024 * 100 * 100;

// 最大频道连接数
pub const MAX_CONNECTIONS: usize = 32;

// 队列最大长度, 超出限制时将抛弃前面的消息
pub const MAX_QUEUE_LENGTH: usize = 1024;
