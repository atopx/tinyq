use std::pin::Pin;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamMap};
use tracing::error;

use crate::{connection::Connection, shutdown::Shutdown, store::Queues};

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
            },
        }
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

/// Stream of messages. The stream receives messages from the
/// `broadcast::Receiver`. We use `stream!` to create a `Stream` that consumes
/// messages. Because `stream!` values cannot be named, we box the stream using
/// a trait object.
type Messages = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

async fn subscribe_to_channel(
    channel_name: String,
    subscriptions: &mut StreamMap<String, Messages>,
    queues: &Queues,
    dst: &mut Connection,
) -> crate::ecode::Result<()> {
    let mut rx = queues.subscribe(channel_name.clone());

    // Subscribe to the channel.
    let rx = Box::pin(async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    println!("{}", String::from_utf8_lossy(&msg));
                    yield msg
                },
                // If we lagged in consuming messages, just resume.
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(_) => break,
            }
        }
    });

    // println!("{:#?}", rx.);

    // Track subscription in this client's subscription set.
    subscriptions.insert(channel_name.clone(), rx);

    // Respond with the successful subscription
    // let response = make_subscribe_frame(channel_name, subscriptions.len());
    // dst.write_frame(&response).await?;

    Ok(())
}
