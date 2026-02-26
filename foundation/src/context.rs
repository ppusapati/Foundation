//! M1: DeterministicContext – the immutable execution context.
//!
//! Carries the logical clock, seed, and configuration snapshot for a
//! deterministic step. Passed by reference to every deterministic operation.

use crate::errors::{FoundationError, FoundationResult};
use crate::time::{LogicalClock, LogicalTimestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Immutable execution context for deterministic operations.
///
/// Encapsulates:
/// - A logical clock for ordering events
/// - A deterministic seed for reproducible randomness
/// - A frozen configuration snapshot
/// - An execution trace for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicContext {
    /// Logical clock for causal ordering.
    clock: LogicalClock,
    /// Deterministic seed (used for reproducible randomness).
    seed: u64,
    /// Frozen configuration key-value pairs.
    config: BTreeMap<String, String>,
    /// Execution trace: operation descriptions appended during a step.
    trace: Vec<TraceEntry>,
    /// Context identifier for audit correlation.
    context_id: String,
}

/// A single entry in the execution trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub timestamp: LogicalTimestamp,
    pub operation: String,
    pub detail: String,
}

impl DeterministicContext {
    /// Create a new context with the given seed.
    pub fn new(seed: u64) -> Self {
        let context_id = format!("ctx-{:016x}", seed);
        DeterministicContext {
            clock: LogicalClock::new(),
            seed,
            config: BTreeMap::new(),
            trace: Vec::new(),
            context_id,
        }
    }

    /// Create a context with an initial configuration.
    pub fn with_config(seed: u64, config: BTreeMap<String, String>) -> Self {
        let context_id = format!("ctx-{:016x}", seed);
        DeterministicContext {
            clock: LogicalClock::new(),
            seed,
            config,
            trace: Vec::new(),
            context_id,
        }
    }

    /// Return the deterministic seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Return the context identifier.
    pub fn context_id(&self) -> &str {
        &self.context_id
    }

    /// Get a configuration value.
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Get a configuration value or a default.
    pub fn get_config_or(&self, key: &str, default: &str) -> String {
        self.config
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Return the current logical timestamp.
    pub fn current_timestamp(&self) -> LogicalTimestamp {
        self.clock.current()
    }

    /// Advance the clock and record a trace entry.
    pub fn tick(&mut self, operation: &str, detail: &str) -> FoundationResult<LogicalTimestamp> {
        let ts = self.clock.tick()?;
        self.trace.push(TraceEntry {
            timestamp: ts,
            operation: operation.to_string(),
            detail: detail.to_string(),
        });
        Ok(ts)
    }

    /// Merge with an external timestamp (e.g., from another system).
    pub fn merge_timestamp(
        &mut self,
        external: &LogicalTimestamp,
    ) -> FoundationResult<LogicalTimestamp> {
        self.clock.merge(external)
    }

    /// Return a snapshot of the trace log.
    pub fn trace(&self) -> &[TraceEntry] {
        &self.trace
    }

    /// Clear the trace (e.g., at the start of a new step).
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    /// Derive a child context with a new sub-seed (deterministic).
    pub fn derive_child(&self, child_index: u64) -> FoundationResult<DeterministicContext> {
        let child_seed = self.seed.checked_add(child_index).ok_or_else(|| {
            FoundationError::Overflow("child seed overflow".into())
        })?;
        let mut child = DeterministicContext::with_config(child_seed, self.config.clone());
        // Child starts from the parent's current clock value
        let parent_ts = self.clock.current();
        if parent_ts.value() > 0 {
            child.clock = LogicalClock::with_initial(parent_ts.value());
        }
        Ok(child)
    }

    /// Produce a simple deterministic pseudo-random u64 from seed + counter.
    /// This is NOT cryptographic – it's for deterministic tie-breaking, etc.
    pub fn deterministic_random(&mut self) -> FoundationResult<u64> {
        let ts = self.clock.tick()?;
        // Simple mixing: seed XOR timestamp value, then multiply by a prime
        let mixed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(ts.value());
        Ok(mixed)
    }
}

impl fmt::Display for DeterministicContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Context(id={}, seed={}, clock={})",
            self.context_id,
            self.seed,
            self.clock.current()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_creation() {
        let ctx = DeterministicContext::new(42);
        assert_eq!(ctx.seed(), 42);
        assert_eq!(ctx.current_timestamp().value(), 0);
    }

    #[test]
    fn context_tick_and_trace() {
        let mut ctx = DeterministicContext::new(42);
        let ts = ctx.tick("test_op", "some detail").unwrap();
        assert_eq!(ts.value(), 1);
        assert_eq!(ctx.trace().len(), 1);
        assert_eq!(ctx.trace()[0].operation, "test_op");
    }

    #[test]
    fn context_config() {
        let mut config = BTreeMap::new();
        config.insert("key1".into(), "value1".into());
        let ctx = DeterministicContext::with_config(0, config);
        assert_eq!(ctx.get_config("key1"), Some(&"value1".to_string()));
        assert_eq!(ctx.get_config("key2"), None);
        assert_eq!(ctx.get_config_or("key2", "default"), "default");
    }

    #[test]
    fn derive_child_context() {
        let mut ctx = DeterministicContext::new(100);
        ctx.tick("parent_op", "").unwrap();
        let child = ctx.derive_child(1).unwrap();
        assert_eq!(child.seed(), 101);
        assert_eq!(child.current_timestamp().value(), 1);
    }

    #[test]
    fn deterministic_random_is_deterministic() {
        let mut ctx1 = DeterministicContext::new(42);
        let mut ctx2 = DeterministicContext::new(42);
        let r1 = ctx1.deterministic_random().unwrap();
        let r2 = ctx2.deterministic_random().unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn clear_trace() {
        let mut ctx = DeterministicContext::new(0);
        ctx.tick("op1", "").unwrap();
        ctx.tick("op2", "").unwrap();
        assert_eq!(ctx.trace().len(), 2);
        ctx.clear_trace();
        assert_eq!(ctx.trace().len(), 0);
    }
}
