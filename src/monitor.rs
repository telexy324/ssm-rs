use tokio::time::{sleep, Duration};

const IDLE_TIMEOUT: i64 = 600;     // 10 min
const MAX_SESSION_TIME: i64 = 7200; // 2 hours

async fn monitor_session(sess: Session) {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(30)) => {
                let now = now_ts();

                if now - sess.started_at > MAX_SESSION_TIME {
                    log::info!("session max time reached {}", sess.id);
                    sess.cancel.notify_waiters();
                    return;
                }

                let last = sess.last_active_at.load(Ordering::Relaxed);
                if now - last > IDLE_TIMEOUT {
                    log::info!("session idle timeout {}", sess.id);
                    sess.cancel.notify_waiters();
                    return;
                }
            }
            _ = sess.cancel.notified() => {
                return;
            }
        }
    }
}
