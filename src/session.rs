use std::{
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Clone)]
struct Session {
    id: String,
    user: String,
    target: String,
    started_at: i64,
    last_active_at: Arc<AtomicI64>,
    cancel: Arc<Notify>,
}

impl Session {
    fn new(user: String, target: String) -> Self {
        let now = now_ts();
        Self {
            id: Uuid::new_v4().to_string(),
            user,
            target,
            started_at: now,
            last_active_at: Arc::new(AtomicI64::new(now)),
            cancel: Arc::new(Notify::new()),
        }
    }

    fn touch(&self) {
        self.last_active_at.store(now_ts(), Ordering::Relaxed);
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
