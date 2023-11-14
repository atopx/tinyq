use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time;
use tracing::{error, info, instrument};

use crate::config::{MAX_BODY_SIZE, MAX_CONNECTIONS};
use crate::store::{Queues, Store};

use super::instruction::{self, ecode, Instruction};

struct Server {
    /// Shared database handle.
    store: Store,
    /// TCP listener supplied by the `run` caller.
    listener: TcpListener,
    /// Limit the max number of connections.
    max_conn: Arc<Semaphore>,
    /// Broadcasts a shutdown signal to all active connections.
    notify_shutdown: broadcast::Sender<()>,
    /// Used as part of the graceful shutdown process to wait for client
    /// connections to complete processing.
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl Server {
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> crate::result::Result<()> {
        info!("accepting inbound connections");

        loop {
            let permit = self.max_conn.clone().acquire_owned().await.unwrap();

            let socket = self.accept().await?;

            let queues = self.store.queues.clone();

            tokio::spawn(async move {
                handle_client(socket, queues).await;
                // Move the permit into the task and drop it after completion.
                // This returns the permit back to the semaphore.
                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> crate::result::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Server {
        listener,
        notify_shutdown,
        shutdown_complete_tx,
        store: Store::new(),
        max_conn: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to tcp accept");
            }
        }
         _ = shutdown => {
            // The shutdown signal has been received.
            info!("server shutting down");
        }
    }

    // Extract the `shutdown_complete` receiver and transmitter
    // explicitly drop `shutdown_transmitter`. This is important, as the
    // `.await` below would otherwise never complete.
    let Server {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    // When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
    // receive the shutdown signal and can exit
    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_complete_tx);

    // Wait for all active connections to finish processing. As the `Sender`
    // handle held by the listener has been dropped above, the only remaining
    // `Sender` instances are held by connection handler tasks. When those drop,
    // the `mpsc` channel will close and `recv()` will return `None`.
    let _ = shutdown_complete_rx.recv().await;
}

async fn handle_client(socket: TcpStream, queues: Queues) {
    let (mut reader, mut writer) = socket.into_split();

    // read instruction
    let instruct: Instruction = if let Ok(v) = reader.read_u8().await {
        v.into()
    } else {
        writer
            .write(&[ecode::INS_PARSE_ERR])
            .await
            .unwrap_or_default();
        return;
    };

    if instruct == instruction::NONE {
        writer
            .write(&[ecode::INS_INVAL_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // read body size
    let body_size: u32 = match reader.read_u32().await {
        Ok(v) => v,
        Err(_) => {
            writer
                .write(&[ecode::BODY_SIZE_PARSE_ERR])
                .await
                .unwrap_or_default();
            return;
        }
    };

    // body_size too large
    if body_size == 0 || body_size > MAX_BODY_SIZE {
        writer
            .write(&[ecode::BODY_SIZE_INVAL_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // read body
    let mut body_buf = vec![0u8; body_size as usize];
    if reader.read(&mut body_buf).await.is_err() {
        writer
            .write(&[ecode::BODY_PARAM_ERR])
            .await
            .unwrap_or_default();
        return;
    }

    // handle instruction
    let (code, size, data) =
        instruction::handle(queues, instruct, body_buf).await;
    writer.write(&[code]).await.unwrap_or_default();
    writer.write(&size.to_be_bytes()).await.unwrap_or_default();
    if size > 0 {
        writer.write_all(&data).await.unwrap_or_default();
    }
}
