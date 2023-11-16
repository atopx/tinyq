use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{config::Mode, connection::Connection, store::Queues};

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
                Err(crate::ecode::ECode::BodyInvalErr)
            },
        }
    }

    pub(crate) async fn apply(
        &self,
        queue: &Queues,
        dst: &mut Connection,
    ) -> crate::ecode::Result<()> {
        match dst.write_code(crate::ecode::ECode::Success).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[clear] reply err {e}");
                Err(crate::ecode::ECode::ServerInternalErr)
            },
        }
    }
}
