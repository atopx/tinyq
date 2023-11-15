use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::sync::Notify;
use tokio::time::{self, Instant};
use tracing::{debug, info};

pub struct Store {
    pub queues: Queues,
}

impl Store {
    pub fn new() -> Self {
        Self {
            queues: Queues::new(),
        }
    }

    pub fn queues(&self) -> Queues {
        self.queues.clone()
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.queues.shutdown_bgtask();
    }
}

#[derive(Debug, Clone)]
pub struct Queues {
    pub shared: Arc<Shared>,
}

impl Queues {
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queues: HashMap::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        });

        // Start the background task.
        tokio::spawn(background_task(shared.clone()));

        Self { shared }
    }

    pub fn push(&self, topic: String, body: Bytes) {
        let mut state = self.shared.state.lock().unwrap();
        let queue =
            state.queues.entry(topic).or_insert_with(|| VecDeque::new());
        if queue.len() >= crate::config::MAX_QUEUE_LENGTH {
            queue.pop_front();
        }
        queue.push_back(body);
    }

    pub fn mpush(&self, topic: String, bodys: Vec<Bytes>) {
        let mut state = self.shared.state.lock().unwrap();
        let queue =
            state.queues.entry(topic).or_insert_with(|| VecDeque::new());
        for body in bodys {
            if queue.len() >= crate::config::MAX_QUEUE_LENGTH {
                // discard the oldest message from the queue.
                queue.pop_back();
            }
            queue.push_front(body);
        }
    }

    pub fn len(&self, topic: String) -> u32 {
        let state = self.shared.state.lock().unwrap();
        match state.queues.get(&topic) {
            Some(q) => q.len() as u32,
            None => 0,
        }
    }

    pub fn pop(&self, topic: String) -> Option<Bytes> {
        let mut state = self.shared.state.lock().unwrap();
        state.queues.get_mut(&topic).and_then(|q| q.pop_back())
    }

    pub fn mpop(&self, topic: String, n: usize) -> Option<Vec<Bytes>> {
        let mut state = self.shared.state.lock().unwrap();
        state
            .queues
            .get_mut(&topic)
            .map(|q| q.drain(q.len().saturating_sub(n)..).collect())
    }

    pub fn clear(&self, topic: String) {
        let mut state = self.shared.state.lock().unwrap();
        state.queues.entry(topic).and_modify(|q| q.clear());
    }

    pub fn del(&self, topic: String) {
        let mut state = self.shared.state.lock().unwrap();
        state.queues.remove(&topic);
    }

    pub fn shutdown_bgtask(&self) {
        let mut state = self.shared.state.lock().unwrap();
        state.shutdown = true;
        drop(state);
        self.shared.background_task.notify_one();
    }
}

#[derive(Debug)]
pub struct State {
    pub queues: HashMap<String, VecDeque<Bytes>>,
    pub shutdown: bool,
}

#[derive(Debug)]
pub struct Shared {
    /// The shared state is guarded by a mutex. This is a `std::sync::Mutex` and
    /// not a Tokio mutex. This is because there are no asynchronous operations
    /// being performed while holding the mutex. Additionally, the critical
    /// sections are very small.
    state: Mutex<State>,

    /// Notifies the background task handling entry expiration. The background
    /// task waits on this to be notified, then checks for expired values or the
    /// shutdown signal.
    background_task: Notify,
}

impl Shared {
    fn loop_bgtask(&self) -> Option<Instant> {
        let state = self.state.lock().unwrap();
        if state.shutdown {
            // The database is shutting down. All handles to the shared state
            // have dropped. The background task should exit.
            return None;
        }
        // loop background task
        None
    }

    /// Returns `true` if the database is shutting down
    ///
    /// The `shutdown` flag is set when all `Db` values have dropped, indicating
    /// that the shared state can no longer be accessed.
    fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }
}

async fn background_task(shared: Arc<Shared>) {
    // If the shutdown flag is set, then the task should exit.
    while !shared.is_shutdown() {
        info!("background task start");
        if let Some(when) = shared.loop_bgtask() {
            // Wait until the next task **or** until the background task
            // This is done by looping.
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            // Wait until the task is notified.
            shared.background_task.notified().await;
        }
    }
    info!("background task shut down")
}
