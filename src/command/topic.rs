use bytes::Bytes;
use serde::Deserialize;
use serde::Serialize;
use tracing::error;

use crate::config::Mode;
use crate::connection::Connection;
use crate::store::Queues;

#[derive(Debug)]
pub struct Action {
    mode: Mode,
    topic: String,
}

impl Action {
    pub async fn new(mode: Mode, body: Bytes) -> crate::ecode::Result<Self> {
        match String::from_utf8(body.to_vec()) {
            Ok(topic) => Ok(Self { mode, topic }),
            Err(e) => {
                error!("[clear] parse err {e}");
                Err(crate::ecode::StatusCode::BodyInvalErr)
            }
        }
    }

    pub(crate) async fn apply(&self, queue: &Queues, dst: &mut Connection) -> crate::ecode::Result<()> {
        match dst.write_code(crate::ecode::StatusCode::Success).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[clear] reply err {e}");
                Err(crate::ecode::StatusCode::ServerInternalErr)
            }
        }
    }
}
