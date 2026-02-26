//! M15: Audit Trail – structured audit logging for deterministic operations.

use crate::time::{LogicalTimestamp, WallTimestamp};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuditLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditLevel::Trace => write!(f, "TRACE"),
            AuditLevel::Debug => write!(f, "DEBUG"),
            AuditLevel::Info => write!(f, "INFO"),
            AuditLevel::Warn => write!(f, "WARN"),
            AuditLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// A single audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Logical timestamp when the event occurred.
    pub logical_ts: LogicalTimestamp,
    /// Wall-clock timestamp (for human consumption only).
    pub wall_ts: WallTimestamp,
    /// Severity level.
    pub level: AuditLevel,
    /// The system/module that produced this event.
    pub source: String,
    /// Short operation name.
    pub operation: String,
    /// Human-readable detail message.
    pub message: String,
    /// Optional context ID for correlation.
    pub context_id: Option<String>,
    /// Optional key-value metadata.
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
}

impl AuditEvent {
    /// Create a new audit event with the current wall clock time.
    pub fn new(
        logical_ts: LogicalTimestamp,
        level: AuditLevel,
        source: &str,
        operation: &str,
        message: &str,
    ) -> Self {
        AuditEvent {
            logical_ts,
            wall_ts: WallTimestamp::now(),
            level,
            source: source.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            context_id: None,
            metadata: None,
        }
    }

    /// Attach a context ID for correlation.
    pub fn with_context_id(mut self, context_id: &str) -> Self {
        self.context_id = Some(context_id.to_string());
        self
    }

    /// Attach metadata.
    pub fn with_metadata(mut self, metadata: std::collections::BTreeMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} {} ({}) – {}",
            self.level, self.logical_ts, self.source, self.operation, self.message
        )
    }
}

/// An append-only audit log that collects events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog { events: Vec::new() }
    }

    /// Append an event to the log.
    pub fn record(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// Return all events.
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Return events filtered by level.
    pub fn events_at_level(&self, level: AuditLevel) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.level == level).collect()
    }

    /// Return events filtered by minimum level.
    pub fn events_at_min_level(&self, min_level: AuditLevel) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.level >= min_level)
            .collect()
    }

    /// Return events matching a context ID.
    pub fn events_for_context(&self, context_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.context_id.as_deref() == Some(context_id))
            .collect()
    }

    /// Number of events in the log.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_creation() {
        let ts = LogicalTimestamp::new(1);
        let event = AuditEvent::new(ts, AuditLevel::Info, "test", "op", "hello");
        assert_eq!(event.level, AuditLevel::Info);
        assert_eq!(event.source, "test");
    }

    #[test]
    fn audit_log_record_and_filter() {
        let mut log = AuditLog::new();
        log.record(AuditEvent::new(
            LogicalTimestamp::new(1),
            AuditLevel::Info,
            "src",
            "op",
            "info msg",
        ));
        log.record(AuditEvent::new(
            LogicalTimestamp::new(2),
            AuditLevel::Error,
            "src",
            "op",
            "error msg",
        ));
        assert_eq!(log.len(), 2);
        assert_eq!(log.events_at_level(AuditLevel::Error).len(), 1);
        assert_eq!(log.events_at_min_level(AuditLevel::Info).len(), 2);
    }

    #[test]
    fn audit_log_context_filter() {
        let mut log = AuditLog::new();
        log.record(
            AuditEvent::new(LogicalTimestamp::new(1), AuditLevel::Info, "s", "o", "m")
                .with_context_id("ctx-1"),
        );
        log.record(AuditEvent::new(
            LogicalTimestamp::new(2),
            AuditLevel::Info,
            "s",
            "o",
            "m",
        ));
        assert_eq!(log.events_for_context("ctx-1").len(), 1);
    }

    #[test]
    fn audit_level_ordering() {
        assert!(AuditLevel::Trace < AuditLevel::Debug);
        assert!(AuditLevel::Debug < AuditLevel::Info);
        assert!(AuditLevel::Info < AuditLevel::Warn);
        assert!(AuditLevel::Warn < AuditLevel::Error);
    }
}
