use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{connection::Connection, shutdown::Shutdown, store::Queues};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {}

impl Action {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) async fn parse(&self, body: Bytes) -> crate::ecode::Result<()> {
        println!("COMMAND auth {}", String::from_utf8(body.to_vec()).unwrap());
        Ok(())
    }

    pub(crate) async fn apply(
        &self,
        queue: &Queues,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::ecode::Result<()> {
        Ok(())
    }
}
