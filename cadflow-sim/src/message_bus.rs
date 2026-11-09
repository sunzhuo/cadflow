//! Discrete event message bus for inter-agent and planner communication.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub from: u32,
    pub to: u32,
    pub action: String,
    pub payload: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MessageBus {
    queue: Vec<MessageEnvelope>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Enqueue an asynchronous message to be dispatched during cycle flush.
    pub fn enqueue(&mut self, from: u32, to: u32, action: impl Into<String>, payload: Option<String>) {
        self.queue.push(MessageEnvelope {
            from,
            to,
            action: action.into(),
            payload,
        });
    }

    /// Drain all pending messages from the queue.
    pub fn flush(&mut self) -> Vec<MessageEnvelope> {
        std::mem::take(&mut self.queue)
    }

    /// Check if queue has pending messages.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Count of pending messages.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Clear all pending messages.
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_bus() {
        let mut bus = MessageBus::new();
        bus.enqueue(1, 2, "query_state", None);
        bus.enqueue(2, 1, "reply_state", Some("idle".into()));

        assert_eq!(bus.len(), 2);
        let flushed = bus.flush();
        assert_eq!(flushed.len(), 2);
        assert!(bus.is_empty());
    }
}
