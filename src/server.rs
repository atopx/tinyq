use std::{future::Future, sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc, Semaphore},
    time,
};
use tracing::{error, info, instrument};

use crate::{
    config::MAX_CONNECTIONS,
    connection::Connection,
    shutdown::Shutdown,
    store::{Queues, Store},
};

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
    pub async fn run(&mut self) -> crate::ecode::Result<()> {
        info!("accepting inbound connections");

        loop {
            // Wait for a permit to become available
            let permit = self.max_conn.clone().acquire_owned().await.unwrap();

            // Accept a new socket. This will attempt to perform error handling.
            // The `accept` method internally attempts to recover errors, so an
            // error here is non-recoverable.
            let socket = self.accept().await?;

            let mut handler = Handler {
                queues: self.store.queues(),
                connect: Connection::new(socket),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                // Process the connection. If an error is encountered, log it.
                if let Err(err) = handler.run().await {
                    error!("connection reset peer");
                    let _ = handler.connect.write_code(err).await;
                }
                // Move the permit into the task and drop it after completion.
                // This returns the permit back to the semaphore.
                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> crate::ecode::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) =>
                    if backoff > 64 {
                        error!("{}", err);
                        return Err(crate::ecode::ECode::ServerBusy);
                    },
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
    let Server { shutdown_complete_tx, notify_shutdown, .. } = server;

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

#[derive(Debug)]
struct Handler {
    queues: Queues,
    connect: Connection,
    shutdown: Shutdown,
    /// Not used directly. Instead, when `Handler` is dropped...?
    _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    #[instrument(skip(self))]
    async fn run(&mut self) -> crate::ecode::Result<()> {
        while !self.shutdown.is_shutdown() {
            self.connect.auth().await?;
            let cmd = tokio::select! {
                res = self.connect.read_command() => res?,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task terminating.
                    return Ok(());
                }
            };

            cmd.apply(&self.queues, &mut self.connect, &mut self.shutdown).await?;
        }

        Ok(())
    }
}
