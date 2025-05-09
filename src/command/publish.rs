use bytes::Bytes;
use tracing::error;

use crate::connection::Connection;
use crate::store::Queues;

#[derive(Debug)]
pub struct Action {
    topic: String,
    body: Bytes,
}

impl Action {
    // publish topic message
    pub async fn new(body: Bytes) -> crate::ecode::Result<Self> {
        let idx = match body.iter().position(|&x| x == 0x20_u8) {
            Some(i) => i,
            None => return Err(crate::ecode::StatusCode::BodyInvalErr),
        };
        let (a, b) = body.split_at(idx + 1);
        let body = Bytes::from(b.to_vec());
        match String::from_utf8(a.to_vec()) {
            Ok(topic) => Ok(Self { topic, body }),
            Err(e) => {
                error!("[publish] parse err {e}");
                Err(crate::ecode::StatusCode::BodyInvalErr)
            }
        }
    }

    pub(crate) async fn apply(&self, queue: &Queues, dst: &mut Connection) -> crate::ecode::Result<()> {
        // queue.push(self.topic.clone(), self.body.clone());
        queue.publish(&self.topic, self.body.clone());
        match dst.write_code(crate::ecode::StatusCode::Success).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[publish] reply err {e}");
                Err(crate::ecode::StatusCode::ServerInternalErr)
            }
        }
    }
}
