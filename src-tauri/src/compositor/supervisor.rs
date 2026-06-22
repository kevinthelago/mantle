use std::future::Future;
use std::time::Duration;

use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use super::event::CompositorEvent;

/// Capacity of the internal broadcast channel.
const BROADCAST_CAP: usize = 512;

/// Maximum backoff between reconnect attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Wraps an event source with automatic reconnection and exponential backoff.
///
/// The supervisor spawns a background task that:
/// 1. Calls `connect` to obtain a stream of events.
/// 2. Forwards events to a broadcast channel.
/// 3. On stream exhaustion (compositor disconnect): emits [`CompositorEvent::Disconnected`],
///    waits with backoff, emits [`CompositorEvent::Reconnecting`], and retries.
///
/// Callers subscribe via [`Supervisor::subscribe`]; each subscriber gets an
/// independent, buffered view of the broadcast channel.
pub struct Supervisor {
    tx: broadcast::Sender<CompositorEvent>,
}

impl Supervisor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAP);
        Self { tx }
    }

    /// Spawn the supervision loop.
    ///
    /// `connect` is an async factory that returns `Some(stream)` on success or
    /// `None` on transient failure (the supervisor will backoff and retry).
    pub fn spawn<F, Fut, S>(&self, connect: F) -> JoinHandle<()>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = Option<S>> + Send,
        S: futures::Stream<Item = CompositorEvent> + Send + Unpin + 'static,
    {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut attempt = 0u32;
            loop {
                match connect().await {
                    Some(mut stream) => {
                        attempt = 0;
                        let _ = tx.send(CompositorEvent::Connected);
                        info!("compositor connected");

                        while let Some(event) = stream.next().await {
                            if tx.send(event).is_err() {
                                // All receivers dropped; nothing to fan out to.
                                return;
                            }
                        }

                        let _ = tx.send(CompositorEvent::Disconnected);
                        warn!("compositor stream ended");
                    }
                    None => {
                        warn!("compositor connect() returned None");
                    }
                }

                attempt = attempt.saturating_add(1);
                let delay = backoff(attempt);
                let _ = tx.send(CompositorEvent::Reconnecting { attempt });
                warn!(attempt, ?delay, "reconnecting after backoff");
                tokio::time::sleep(delay).await;
            }
        })
    }

    /// Create a new subscriber stream.
    ///
    /// Events that arrive while the subscriber's internal buffer is full are
    /// silently dropped (the broadcast channel raises `RecvError::Lagged`).
    pub fn subscribe(&self) -> BoxStream<'static, CompositorEvent> {
        let mut rx = self.tx.subscribe();
        Box::pin(async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(event) => yield event,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(n, "supervisor: subscriber lagged, events dropped");
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
    }

    /// Return a clone of the raw sender (used by backends to feed extra events,
    /// e.g. `Connected` / `Disconnected` from a separate health-check path).
    pub fn sender(&self) -> broadcast::Sender<CompositorEvent> {
        self.tx.clone()
    }
}

/// Exponential backoff capped at [`MAX_BACKOFF`].
///
/// attempt 1 → 200 ms, 2 → 400 ms, 3 → 800 ms … 7+ → 30 s
fn backoff(attempt: u32) -> Duration {
    let millis = 100u64 * 2u64.pow(attempt.min(8));
    Duration::from_millis(millis).min(MAX_BACKOFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_caps_at_max() {
        assert!(backoff(20) <= MAX_BACKOFF);
    }

    #[test]
    fn backoff_increases() {
        assert!(backoff(1) < backoff(2));
        assert!(backoff(2) < backoff(3));
    }
}
