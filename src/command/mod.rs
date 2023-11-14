use bytes::Bytes;

use crate::{connection::Connection, shutdown::Shutdown, store::Queues};

pub mod auth;
pub mod clear;
pub mod delete;
pub mod publish;
pub mod ready;
pub mod subscribe;

#[derive(Debug)]
pub enum Command {
    Auth(auth::Action),
    Publish(publish::Action),
    Subscribe(subscribe::Action),
    Ready(ready::Action),
    Clear(clear::Action),
    Delete(delete::Action),
}

impl Command {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Command::Auth(auth::Action::new())),
            2 => Some(Command::Publish(publish::Action::new())),
            3 => Some(Command::Subscribe(subscribe::Action::new())),
            4 => Some(Command::Ready(ready::Action::new())),
            6 => Some(Command::Clear(clear::Action::new())),
            7 => Some(Command::Delete(delete::Action::new())),
            _ => None,
        }
    }

    pub(crate) async fn parse(&self, body: Bytes) -> crate::ecode::Result<()> {
        match self {
            Self::Auth(cmd) => cmd.parse(body).await,
            Self::Publish(cmd) => cmd.parse(body).await,
            Self::Subscribe(cmd) => cmd.parse(body).await,
            Self::Ready(cmd) => cmd.parse(body).await,
            Self::Clear(cmd) => cmd.parse(body).await,
            Self::Delete(cmd) => cmd.parse(body).await,
        }
    }

    pub(crate) async fn apply(
        &self,
        queues: &Queues,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::ecode::Result<()> {
        match self {
            Self::Auth(cmd) => cmd.apply(queues, dst).await,
            Self::Publish(cmd) => cmd.apply(queues, dst).await,
            Self::Subscribe(cmd) => cmd.apply(queues, dst, shutdown).await,
            Self::Ready(cmd) => cmd.apply(queues, dst).await,
            Self::Clear(cmd) => cmd.apply(queues, dst).await,
            Self::Delete(cmd) => cmd.apply(queues, dst).await,
        }
    }
}

trait CommandAction {
    fn parse(&self, body: Bytes) -> crate::ecode::Result<()> {
        println!("COMMAND auth {}", String::from_utf8(body.to_vec()).unwrap());
        Ok(())
    }

    fn apply(queue: &Queues, dst: &mut Connection) -> crate::ecode::Result<()> {
        Ok(())
    }
}
