use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{connection::Connection, store::Queues};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    topic: String,
}

impl Action {
    pub async fn new(body: Bytes) -> crate::ecode::Result<Self> {
        match String::from_utf8(body.to_vec()) {
            Ok(topic) => Ok(Self { topic }),
            Err(e) => {
                error!("[clear] parse err {e}");
                Err(crate::ecode::ECode::BodyInvalErr)
            }
        }
    }

    pub(crate) async fn apply(
        &self,
        queue: &Queues,
        dst: &mut Connection,
    ) -> crate::ecode::Result<()> {
        queue.clear(self.topic.to_owned());
        Ok(())
    }
}
