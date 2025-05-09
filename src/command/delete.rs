use bytes::Bytes;
use serde::Deserialize;
use serde::Serialize;
use tracing::error;

use crate::connection::Connection;
use crate::store::Queues;

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    topic: String,
}

impl Action {
    pub async fn new(body: Bytes) -> crate::ecode::Result<Self> {
        match String::from_utf8(body.to_vec()) {
            Ok(topic) => Ok(Self { topic }),
            Err(e) => {
                error!("[delete] parse err {e}");
                Err(crate::ecode::StatusCode::BodyInvalErr)
            }
        }
    }

    pub(crate) async fn apply(&self, queue: &Queues, dst: &mut Connection) -> crate::ecode::Result<()> {
        queue.del(self.topic.to_owned());
        match dst.write_code(crate::ecode::StatusCode::Success).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[delete] reply err {e}");
                Err(crate::ecode::StatusCode::ServerInternalErr)
            }
        }
    }
}
