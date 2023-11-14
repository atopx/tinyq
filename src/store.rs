use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::sync::Notify;
use tokio::time::{self, Instant};
use tracing::debug;

pub struct Store {
    pub queues: Queues,
}

impl Store {
    pub fn new() -> Self {
        Self {
            queues: Queues::new(),
        }
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

    pub fn push(&mut self, topic: String, message: Message) {
        let mut state = self.shared.state.lock().unwrap();
        let queue = state
            .queues
            .entry(topic)
            .or_insert_with(|| BinaryHeap::new());
        if queue.len() >= crate::config::MAX_QUEUE_LENGTH {
            // todo heap overflow?
        }
        queue.push(message);
    }

    pub fn len(&mut self, topic: String) -> u32 {
        let state = self.shared.state.lock().unwrap();
        match state.queues.get(&topic) {
            Some(q) => q.len() as u32,
            None => 0,
        }
    }

    pub fn pop(&mut self, topic: String) -> Option<Message> {
        let mut state = self.shared.state.lock().unwrap();
        state.queues.get_mut(&topic).and_then(|q| q.pop())
    }

    pub fn clear(&mut self, topic: String) {
        let mut state = self.shared.state.lock().unwrap();
        state.queues.entry(topic).and_modify(|q| q.clear());
    }

    pub fn del(&mut self, topic: String) {
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
    pub queues: HashMap<String, BinaryHeap<Message>>,
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

    debug!("Purge background task shut down")
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub body: Bytes,
    pub priority: u8,
}

// Implement Ord and PartialOrd traits for Message based on priority
impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sorting by priority in reverse order
        self.priority.cmp(&other.priority).reverse()
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
