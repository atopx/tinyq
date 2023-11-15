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
    pub async fn new(v: u8, body: Bytes) -> crate::ecode::Result<Self> {
        match v {
            1 => Ok(Command::Auth(auth::Action::new(body).await?)),
            2 => Ok(Command::Publish(publish::Action::new(body).await?)),
            3 => Ok(Command::Subscribe(subscribe::Action::new(body).await?)),
            4 => Ok(Command::Ready(ready::Action::new(body).await?)),
            6 => Ok(Command::Clear(clear::Action::new(body).await?)),
            7 => Ok(Command::Delete(delete::Action::new(body).await?)),
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
