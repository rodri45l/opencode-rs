//! In-process publish/subscribe for server events.
//!
//! Backed by a `tokio::sync::broadcast` channel. Slow subscribers lag rather
//! than block publishers, matching the reference server's SSE behaviour.

use opencode_schema::EventEnvelope;
use tokio::sync::broadcast;

/// A receiver of published events.
pub struct EventSubscription {
    rx: broadcast::Receiver<EventEnvelope>,
}

impl EventSubscription {
    /// Await the next event.
    ///
    /// Returns `None` once the bus is dropped.
    pub async fn recv(&mut self) -> Option<EventEnvelope> {
        loop {
            match self.rx.recv().await {
                Ok(event) => return Some(event),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(skipped, "event subscriber lagged");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

/// Publishes and subscribes to server events.
pub trait EventBus: Send + Sync + 'static {
    /// Publish an event to all current subscribers.
    fn publish(&self, event: EventEnvelope);

    /// Subscribe to future events.
    fn subscribe(&self) -> EventSubscription;
}

/// Broadcast-backed event bus.
pub struct InMemoryEventBus {
    tx: broadcast::Sender<EventEnvelope>,
}

impl InMemoryEventBus {
    /// Create a bus retaining up to `capacity` events for slow subscribers.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: EventEnvelope) {
        // Err only when there are no subscribers, which is not an error.
        let _ = self.tx.send(event);
    }

    fn subscribe(&self) -> EventSubscription {
        EventSubscription {
            rx: self.tx.subscribe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_schema::EventType;

    #[tokio::test]
    async fn delivers_published_events() {
        let bus = InMemoryEventBus::default();
        let mut sub = bus.subscribe();
        let event = EventEnvelope::new(EventType::SessionCreated, serde_json::json!({}));
        bus.publish(event.clone());
        assert_eq!(sub.recv().await.unwrap().id, event.id);
    }

    #[tokio::test]
    async fn publishes_without_subscribers() {
        let bus = InMemoryEventBus::default();
        bus.publish(EventEnvelope::new(
            EventType::SessionCreated,
            serde_json::json!({}),
        ));
    }
}
