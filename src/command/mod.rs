use bytes::Bytes;

use crate::{config::Mode, connection::Connection, shutdown::Shutdown, store::Queues};

pub mod clear;
pub mod delete;
pub mod publish;
pub mod subscribe;
mod topic;

#[derive(Debug)]
pub enum Command {
    Topic(topic::Action),
    Publish(publish::Action),
    Subscribe(subscribe::Action),
    Clear(clear::Action),
    Delete(delete::Action),
}

impl Command {
    pub async fn new(v: u8, body: Bytes) -> crate::ecode::Result<Self> {
        match v {
            // create a consume mode queue, format: `1 BODY_SIZE TOPIC`, example: 111first_topic
            1 => Ok(Command::Topic(topic::Action::new(Mode::Consume, body).await?)),
            // create a broadcast mode queue, format: `2 BODY_SIZE TOPIC`, example: 211first_topic
            2 => Ok(Command::Topic(topic::Action::new(Mode::Broadcast, body).await?)),
            // publish a message to queue, format: `3 BODY_SIZE TOPIC BODY`, example: 31120first_topic {"a": 1}
            3 => Ok(Command::Publish(publish::Action::new(body).await?)),
            // subscribe a message to queue, format: `4 BODY_SIZE TOPIC`, example: 411first_topic
            4 => Ok(Command::Subscribe(subscribe::Action::new(body).await?)),
            // clear a queue, format: `200 BODY_SIZE TOPIC`, example: 20011first_topic
            200 => Ok(Command::Clear(clear::Action::new(body).await?)),
            // delete a queue, format: `201 BODY_SIZE TOPIC`, example: 20111first_topic
            201 => Ok(Command::Delete(delete::Action::new(body).await?)),
            _ => Err(crate::ecode::ECode::CmdInvalErr),
        }
    }

    pub(crate) async fn apply(
        &self,
        queues: &Queues,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::ecode::Result<()> {
        match self {
            Self::Topic(cmd) => cmd.apply(queues, dst).await,
            Self::Publish(cmd) => cmd.apply(queues, dst).await,
            Self::Subscribe(cmd) => cmd.apply(queues, dst, shutdown).await,
            Self::Clear(cmd) => cmd.apply(queues, dst).await,
            Self::Delete(cmd) => cmd.apply(queues, dst).await,
        }
    }
}
