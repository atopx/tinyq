use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use tokio::{
    sync::{broadcast, mpsc, Notify},
    time::{self, Instant},
};
use tracing::{debug, info};

use crate::config::MAX_QUEUE_LENGTH;

pub struct Store {
    pub queues: Queues,
}

impl Store {
    pub fn new() -> Self {
        Self { queues: Queues::new() }
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
                channels: HashMap::new(),
            }),
            background_task: Notify::new(),
        });

        // Start the background task.
        tokio::spawn(background_task(shared.clone()));

        Self { shared }
    }

    pub fn push(&self, topic: String, body: Bytes) {
        let mut state = self.shared.state.lock().unwrap();
        let queue = state.queues.entry(topic).or_insert_with(|| VecDeque::new());
        if queue.len() >= crate::config::MAX_QUEUE_LENGTH {
            queue.pop_front();
        }
        queue.push_back(body);
    }

    pub fn mpush(&self, topic: String, bodys: Vec<Bytes>) {
        let mut state = self.shared.state.lock().unwrap();
        let queue = state.queues.entry(topic).or_insert_with(|| VecDeque::new());
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

    /// 将消息发送到通道。返回订阅该通道的订阅者数量。
    pub(crate) fn publish(&self, topic: &str, body: Bytes) -> usize {
        let state = self.shared.state.lock().unwrap();

        state
            .channels
            .get(topic)
            // 在成功将消息发送到广播通道后，返回订阅者数量。
            // 如果发生错误，表示没有接收者，应返回0。
            .map(|tx| tx.send(body).unwrap_or(0))
            // 如果通道键没有找到，则没有订阅者。在这种情况下，返回0。
            .unwrap_or(0)
    }

    /// 返回一个用于指定通道的 `Receiver`。
    /// 返回的 `Receiver` 用于接收通过 `PUBLISH` 命令广播的值
    pub(crate) fn subscribe(&self, topic: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;

        // Acquire the mutex
        let mut state = self.shared.state.lock().unwrap();

        // 如果请求的通道不存在，则创建一个新的广播通道并将其与键关联。
        // 如果已经存在，则返回与该通道关联的接收者。
        match state.channels.entry(topic) {
            Entry::Occupied(e) => e.get().subscribe(),
            Entry::Vacant(e) => {
                // 还没有与请求的通道关联的广播通道，所以创建一个新广播通道并将其与键关联。
                // 广播通道的容量为 `MAX_QUEUE_LENGTH`
                // 条消息。消息存储在通道中，直到所有订阅者都看到它。
                // 这意味着一个慢订阅者可能会导致消息被无限期地保留在通道中。
                // 当广播通道的容量满时，发布将导致旧消息被丢弃。这将阻止慢消费者阻止整个系统。
                let (tx, rx) = broadcast::channel(MAX_QUEUE_LENGTH);
                e.insert(tx);
                rx
            },
        }
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

        // Drop the lock before signalling the background task. This helps
        // reduce lock contention by ensuring the background task doesn't
        // wake up only to be unable to acquire the mutex.
        drop(state);
        self.shared.background_task.notify_one();
    }
}

#[derive(Debug)]
pub struct State {
    pub queues: HashMap<String, VecDeque<Bytes>>,
    pub channels: HashMap<String, broadcast::Sender<Bytes>>,
    pub shutdown: bool,
}

#[derive(Debug)]
pub struct Shared {
    /// 共享状态由一个互斥锁保护, 这是一个 `std::sync::Mutex` 而不是一个 Tokio 互斥锁.
    /// 这是因为没有在持有互斥锁时执行任何异步操作。此外，临界区非常小。
    state: Mutex<State>,

    /// 后台任务等待通知，然后执行任务或关闭信号。
    background_task: Notify,
}

impl Shared {
    fn loop_bgtask(&self) -> Option<Instant> {
        let state = self.state.lock().unwrap();
        if state.shutdown {
            // 系统正在关闭。所有共享状态的句柄都已释放
            // todo 退出后台任务。
            return None;
        }
        // todo this run background task.
        // 1. back store timer
        // 2. check server state
        // 3. report server info
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
