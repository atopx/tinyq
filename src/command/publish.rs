use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{connection::Connection, store::Queues};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {}

impl Action {
    pub async fn new(body: Bytes) -> crate::ecode::Result<Self> {
        match String::from_utf8(body.to_vec()) {
            Ok(v) => Ok(Self {}),
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
        Ok(())
    }
}
