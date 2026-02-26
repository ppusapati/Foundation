//! M19-M21: Transport Layer – message types, routing, and client abstractions.
//!
//! Provides a deterministic, type-safe inter-system communication layer.

use crate::errors::{FoundationError, FoundationResult};
use crate::time::LogicalTimestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

// ===========================================================================
// M19: Transport Types
// ===========================================================================

/// A unique identifier for a message.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(String);

impl MessageId {
    pub fn new(id: &str) -> Self {
        MessageId(id.to_string())
    }

    pub fn generate(seed: u64, counter: u64) -> Self {
        MessageId(format!("msg-{:016x}-{:08x}", seed, counter))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies a system in the architecture.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemAddress {
    GraphSystem,
    AgentSystem,
    BlueprintSystem,
    TaskSystem,
    Custom(String),
}

impl fmt::Display for SystemAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemAddress::GraphSystem => write!(f, "graph"),
            SystemAddress::AgentSystem => write!(f, "agent"),
            SystemAddress::BlueprintSystem => write!(f, "blueprint"),
            SystemAddress::TaskSystem => write!(f, "task"),
            SystemAddress::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Priority levels for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// The payload of a transport message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// A request with an operation name and JSON body.
    Request {
        operation: String,
        body: serde_json::Value,
    },
    /// A response to a previous request.
    Response {
        request_id: MessageId,
        status: ResponseStatus,
        body: serde_json::Value,
    },
    /// An event notification.
    Event {
        event_type: String,
        body: serde_json::Value,
    },
}

/// Status of a response message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Failure,
    PartialSuccess,
}

/// A transport message with full routing and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    pub id: MessageId,
    pub source: SystemAddress,
    pub destination: SystemAddress,
    pub payload: MessagePayload,
    pub priority: Priority,
    pub timestamp: LogicalTimestamp,
    pub correlation_id: Option<MessageId>,
    pub headers: BTreeMap<String, String>,
}

impl TransportMessage {
    /// Create a new request message.
    pub fn request(
        id: MessageId,
        source: SystemAddress,
        destination: SystemAddress,
        operation: &str,
        body: serde_json::Value,
        ts: LogicalTimestamp,
    ) -> Self {
        TransportMessage {
            id,
            source,
            destination,
            payload: MessagePayload::Request {
                operation: operation.to_string(),
                body,
            },
            priority: Priority::Normal,
            timestamp: ts,
            correlation_id: None,
            headers: BTreeMap::new(),
        }
    }

    /// Create a response to a given request.
    pub fn response(
        id: MessageId,
        request: &TransportMessage,
        status: ResponseStatus,
        body: serde_json::Value,
        ts: LogicalTimestamp,
    ) -> Self {
        TransportMessage {
            id,
            source: request.destination.clone(),
            destination: request.source.clone(),
            payload: MessagePayload::Response {
                request_id: request.id.clone(),
                status,
                body,
            },
            priority: request.priority,
            timestamp: ts,
            correlation_id: Some(request.id.clone()),
            headers: BTreeMap::new(),
        }
    }

    /// Create an event message.
    pub fn event(
        id: MessageId,
        source: SystemAddress,
        destination: SystemAddress,
        event_type: &str,
        body: serde_json::Value,
        ts: LogicalTimestamp,
    ) -> Self {
        TransportMessage {
            id,
            source,
            destination,
            payload: MessagePayload::Event {
                event_type: event_type.to_string(),
                body,
            },
            priority: Priority::Normal,
            timestamp: ts,
            correlation_id: None,
            headers: BTreeMap::new(),
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Add a header.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

impl fmt::Display for TransportMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Message({}: {} -> {})",
            self.id, self.source, self.destination
        )
    }
}

// ===========================================================================
// M20: Router
// ===========================================================================

/// A handler function type for processing transport messages.
pub type MessageHandler = Box<dyn Fn(&TransportMessage) -> FoundationResult<TransportMessage> + Send + Sync>;

/// A route entry mapping (source, destination, operation pattern) to a handler index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub source: Option<SystemAddress>,
    pub destination: SystemAddress,
    pub operation_pattern: Option<String>,
    pub handler_name: String,
}

/// A deterministic message router.
///
/// Routes transport messages to registered handlers based on destination
/// and optional operation pattern matching.
pub struct MessageRouter {
    routes: Vec<(RouteEntry, MessageHandler)>,
    undelivered: Vec<TransportMessage>,
}

impl MessageRouter {
    pub fn new() -> Self {
        MessageRouter {
            routes: Vec::new(),
            undelivered: Vec::new(),
        }
    }

    /// Register a route with a handler.
    pub fn register(
        &mut self,
        destination: SystemAddress,
        operation_pattern: Option<String>,
        handler_name: &str,
        handler: MessageHandler,
    ) {
        let entry = RouteEntry {
            source: None,
            destination,
            operation_pattern,
            handler_name: handler_name.to_string(),
        };
        self.routes.push((entry, handler));
    }

    /// Route a message to the appropriate handler.
    pub fn route(&mut self, message: &TransportMessage) -> FoundationResult<TransportMessage> {
        let operation = match &message.payload {
            MessagePayload::Request { operation, .. } => Some(operation.as_str()),
            MessagePayload::Event { event_type, .. } => Some(event_type.as_str()),
            MessagePayload::Response { .. } => None,
        };

        for (entry, handler) in &self.routes {
            if entry.destination != message.destination {
                continue;
            }
            if let Some(pattern) = &entry.operation_pattern {
                if let Some(op) = operation {
                    if op != pattern {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            return handler(message);
        }

        self.undelivered.push(message.clone());
        Err(FoundationError::TransportError(format!(
            "no route for message {} to {}",
            message.id, message.destination
        )))
    }

    /// Return undelivered messages.
    pub fn undelivered(&self) -> &[TransportMessage] {
        &self.undelivered
    }

    /// Clear undelivered messages.
    pub fn clear_undelivered(&mut self) {
        self.undelivered.clear();
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// M21: Client Abstractions
// ===========================================================================

/// A trait for system clients that can send and receive transport messages.
pub trait TransportClient: Send + Sync {
    /// Send a message and receive a response.
    fn send(&self, message: TransportMessage) -> FoundationResult<TransportMessage>;

    /// The system address of this client.
    fn address(&self) -> &SystemAddress;
}

/// A simple in-memory loopback client for testing.
pub struct LoopbackClient {
    address: SystemAddress,
    response_fn: Box<dyn Fn(&TransportMessage) -> TransportMessage + Send + Sync>,
}

impl LoopbackClient {
    pub fn new(
        address: SystemAddress,
        response_fn: Box<dyn Fn(&TransportMessage) -> TransportMessage + Send + Sync>,
    ) -> Self {
        LoopbackClient {
            address,
            response_fn,
        }
    }
}

impl TransportClient for LoopbackClient {
    fn send(&self, message: TransportMessage) -> FoundationResult<TransportMessage> {
        Ok((self.response_fn)(&message))
    }

    fn address(&self) -> &SystemAddress {
        &self.address
    }
}

/// A message queue for buffered, ordered delivery.
#[derive(Debug, Default)]
pub struct MessageQueue {
    queue: std::collections::VecDeque<TransportMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            queue: std::collections::VecDeque::new(),
        }
    }

    /// Enqueue a message.
    pub fn enqueue(&mut self, message: TransportMessage) {
        self.queue.push_back(message);
    }

    /// Dequeue the next message.
    pub fn dequeue(&mut self) -> Option<TransportMessage> {
        self.queue.pop_front()
    }

    /// Peek at the next message without removing it.
    pub fn peek(&self) -> Option<&TransportMessage> {
        self.queue.front()
    }

    /// Number of messages in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Drain all messages as a Vec, sorted by priority (highest first).
    pub fn drain_by_priority(&mut self) -> Vec<TransportMessage> {
        let mut messages: Vec<_> = self.queue.drain(..).collect();
        messages.sort_by(|a, b| b.priority.cmp(&a.priority));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_creation() {
        let msg = TransportMessage::request(
            MessageId::new("msg-1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "create_node",
            serde_json::json!({"label": "test"}),
            LogicalTimestamp::new(1),
        );
        assert_eq!(msg.source, SystemAddress::AgentSystem);
        assert_eq!(msg.destination, SystemAddress::GraphSystem);
    }

    #[test]
    fn message_response() {
        let req = TransportMessage::request(
            MessageId::new("msg-1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "query",
            serde_json::json!({}),
            LogicalTimestamp::new(1),
        );
        let resp = TransportMessage::response(
            MessageId::new("msg-2"),
            &req,
            ResponseStatus::Success,
            serde_json::json!({"result": "ok"}),
            LogicalTimestamp::new(2),
        );
        assert_eq!(resp.destination, SystemAddress::AgentSystem);
        assert_eq!(resp.correlation_id, Some(MessageId::new("msg-1")));
    }

    #[test]
    fn router_routes_to_handler() {
        let mut router = MessageRouter::new();
        router.register(
            SystemAddress::GraphSystem,
            Some("create_node".to_string()),
            "create_handler",
            Box::new(|msg| {
                Ok(TransportMessage::response(
                    MessageId::new("resp-1"),
                    msg,
                    ResponseStatus::Success,
                    serde_json::json!({"created": true}),
                    LogicalTimestamp::new(2),
                ))
            }),
        );

        let msg = TransportMessage::request(
            MessageId::new("msg-1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "create_node",
            serde_json::json!({}),
            LogicalTimestamp::new(1),
        );

        let resp = router.route(&msg).unwrap();
        assert_eq!(resp.destination, SystemAddress::AgentSystem);
    }

    #[test]
    fn router_undelivered() {
        let mut router = MessageRouter::new();
        let msg = TransportMessage::request(
            MessageId::new("msg-1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "unknown_op",
            serde_json::json!({}),
            LogicalTimestamp::new(1),
        );
        assert!(router.route(&msg).is_err());
        assert_eq!(router.undelivered().len(), 1);
    }

    #[test]
    fn message_queue_basic() {
        let mut q = MessageQueue::new();
        assert!(q.is_empty());

        q.enqueue(TransportMessage::event(
            MessageId::new("e1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "tick",
            serde_json::json!({}),
            LogicalTimestamp::new(1),
        ));
        assert_eq!(q.len(), 1);

        let msg = q.dequeue().unwrap();
        assert_eq!(msg.id, MessageId::new("e1"));
        assert!(q.is_empty());
    }

    #[test]
    fn message_queue_priority_drain() {
        let mut q = MessageQueue::new();
        q.enqueue(
            TransportMessage::event(
                MessageId::new("e1"),
                SystemAddress::AgentSystem,
                SystemAddress::GraphSystem,
                "low",
                serde_json::json!({}),
                LogicalTimestamp::new(1),
            )
            .with_priority(Priority::Low),
        );
        q.enqueue(
            TransportMessage::event(
                MessageId::new("e2"),
                SystemAddress::AgentSystem,
                SystemAddress::GraphSystem,
                "critical",
                serde_json::json!({}),
                LogicalTimestamp::new(2),
            )
            .with_priority(Priority::Critical),
        );
        let drained = q.drain_by_priority();
        assert_eq!(drained[0].id, MessageId::new("e2")); // Critical first
        assert_eq!(drained[1].id, MessageId::new("e1")); // Low second
    }

    #[test]
    fn loopback_client() {
        let client = LoopbackClient::new(
            SystemAddress::GraphSystem,
            Box::new(|msg| {
                TransportMessage::response(
                    MessageId::new("resp-1"),
                    msg,
                    ResponseStatus::Success,
                    serde_json::json!({}),
                    LogicalTimestamp::new(2),
                )
            }),
        );

        let msg = TransportMessage::request(
            MessageId::new("msg-1"),
            SystemAddress::AgentSystem,
            SystemAddress::GraphSystem,
            "test",
            serde_json::json!({}),
            LogicalTimestamp::new(1),
        );

        let resp = client.send(msg).unwrap();
        assert_eq!(resp.destination, SystemAddress::AgentSystem);
    }
}
