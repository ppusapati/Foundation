//! M10: Time Helpers – deterministic logical timestamps and clocks.

use crate::errors::{FoundationError, FoundationResult};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// LogicalTimestamp – monotonically increasing sequence counter
// ---------------------------------------------------------------------------

/// A logical (Lamport-style) timestamp represented as a u64 counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LogicalTimestamp(u64);

impl LogicalTimestamp {
    pub const ZERO: Self = LogicalTimestamp(0);

    #[inline]
    pub const fn new(value: u64) -> Self {
        LogicalTimestamp(value)
    }

    #[inline]
    pub const fn value(&self) -> u64 {
        self.0
    }

    /// Increment the timestamp, returning the new value.
    pub fn increment(&self) -> FoundationResult<LogicalTimestamp> {
        self.0
            .checked_add(1)
            .map(LogicalTimestamp)
            .ok_or_else(|| FoundationError::Overflow("logical timestamp overflow".into()))
    }

    /// Merge with another timestamp: max(self, other) + 1.
    pub fn merge(&self, other: &LogicalTimestamp) -> FoundationResult<LogicalTimestamp> {
        let max_val = self.0.max(other.0);
        max_val
            .checked_add(1)
            .map(LogicalTimestamp)
            .ok_or_else(|| FoundationError::Overflow("logical timestamp merge overflow".into()))
    }
}

impl fmt::Display for LogicalTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// LogicalClock – a mutable clock that produces increasing timestamps
// ---------------------------------------------------------------------------

/// A logical clock that produces monotonically increasing timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalClock {
    current: u64,
}

impl LogicalClock {
    pub fn new() -> Self {
        LogicalClock { current: 0 }
    }

    pub fn with_initial(value: u64) -> Self {
        LogicalClock { current: value }
    }

    /// Return the current timestamp without advancing.
    pub fn current(&self) -> LogicalTimestamp {
        LogicalTimestamp(self.current)
    }

    /// Advance and return the next timestamp.
    pub fn tick(&mut self) -> FoundationResult<LogicalTimestamp> {
        self.current = self.current.checked_add(1).ok_or_else(|| {
            FoundationError::Overflow("logical clock overflow".into())
        })?;
        Ok(LogicalTimestamp(self.current))
    }

    /// Merge with an external timestamp: set clock to max(current, external) + 1.
    pub fn merge(&mut self, external: &LogicalTimestamp) -> FoundationResult<LogicalTimestamp> {
        let max_val = self.current.max(external.value());
        self.current = max_val.checked_add(1).ok_or_else(|| {
            FoundationError::Overflow("logical clock merge overflow".into())
        })?;
        Ok(LogicalTimestamp(self.current))
    }
}

impl Default for LogicalClock {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// WallTimestamp – wrapper around chrono for wall clock
// ---------------------------------------------------------------------------

/// A wall clock timestamp (UTC) for audit/logging purposes.
/// Not used for deterministic logic – use `LogicalTimestamp` instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WallTimestamp {
    iso8601: String,
}

impl WallTimestamp {
    /// Capture the current UTC time.
    pub fn now() -> Self {
        let now = chrono::Utc::now();
        WallTimestamp {
            iso8601: now.to_rfc3339(),
        }
    }

    /// Create from an ISO 8601 string.
    pub fn from_iso(s: &str) -> FoundationResult<Self> {
        // Validate by parsing
        chrono::DateTime::parse_from_rfc3339(s).map_err(|e| {
            FoundationError::ValidationFailed(format!("invalid ISO 8601 timestamp: {}", e))
        })?;
        Ok(WallTimestamp {
            iso8601: s.to_string(),
        })
    }

    pub fn as_iso(&self) -> &str {
        &self.iso8601
    }
}

impl fmt::Display for WallTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso8601)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_timestamp_increment() {
        let t = LogicalTimestamp::ZERO;
        let t1 = t.increment().unwrap();
        assert_eq!(t1.value(), 1);
    }

    #[test]
    fn logical_timestamp_merge() {
        let a = LogicalTimestamp::new(5);
        let b = LogicalTimestamp::new(10);
        let merged = a.merge(&b).unwrap();
        assert_eq!(merged.value(), 11);
    }

    #[test]
    fn logical_clock_tick() {
        let mut clock = LogicalClock::new();
        assert_eq!(clock.current().value(), 0);
        let t1 = clock.tick().unwrap();
        assert_eq!(t1.value(), 1);
        let t2 = clock.tick().unwrap();
        assert_eq!(t2.value(), 2);
    }

    #[test]
    fn logical_clock_merge() {
        let mut clock = LogicalClock::new();
        clock.tick().unwrap(); // 1
        let external = LogicalTimestamp::new(10);
        let merged = clock.merge(&external).unwrap();
        assert_eq!(merged.value(), 11);
    }

    #[test]
    fn wall_timestamp_now() {
        let ts = WallTimestamp::now();
        assert!(!ts.as_iso().is_empty());
    }

    #[test]
    fn wall_timestamp_from_iso() {
        let ts = WallTimestamp::from_iso("2024-01-15T10:30:00+00:00").unwrap();
        assert!(ts.as_iso().contains("2024"));
    }

    #[test]
    fn wall_timestamp_invalid() {
        assert!(WallTimestamp::from_iso("not-a-timestamp").is_err());
    }
}
