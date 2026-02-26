//! M22: Generic Deterministic ID Framework.
//!
//! Provides a type-safe, domain-tagged, deterministic ID system.
//! IDs are SHA-256 hashes of `(domain_tag || canonical_bytes)`.

use crate::errors::{FoundationError, FoundationResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Domain tag trait
// ---------------------------------------------------------------------------

/// Trait that domain types implement to provide a unique tag for ID generation.
pub trait DomainTag {
    /// A unique, stable string identifying this domain (e.g. "node", "edge").
    fn tag() -> &'static str;
}

// ---------------------------------------------------------------------------
// DeterministicId<D>
// ---------------------------------------------------------------------------

/// A deterministic, domain-tagged identifier represented as a hex-encoded
/// SHA-256 hash.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeterministicId<D: DomainTag> {
    hex: String,
    #[serde(skip)]
    _marker: PhantomData<D>,
}

impl<D: DomainTag> DeterministicId<D> {
    /// Generate an ID by hashing `domain_tag || content_bytes`.
    pub fn generate(content_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(D::tag().as_bytes());
        hasher.update(content_bytes);
        let hash = hasher.finalize();
        DeterministicId {
            hex: hex_encode(&hash),
            _marker: PhantomData,
        }
    }

    /// Generate an ID from a string representation of the content.
    pub fn from_content(content: &str) -> Self {
        Self::generate(content.as_bytes())
    }

    /// Generate an ID from multiple byte slices concatenated together.
    pub fn from_parts(parts: &[&[u8]]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(D::tag().as_bytes());
        for part in parts {
            hasher.update(part);
        }
        let hash = hasher.finalize();
        DeterministicId {
            hex: hex_encode(&hash),
            _marker: PhantomData,
        }
    }

    /// Create a `DeterministicId` from an existing hex string.
    /// Validates that the string is a 64-character hex string.
    pub fn from_hex(hex: &str) -> FoundationResult<Self> {
        if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(FoundationError::IdError(
                "ID must be a 64-character hex string".into(),
            ));
        }
        Ok(DeterministicId {
            hex: hex.to_lowercase(),
            _marker: PhantomData,
        })
    }

    /// Return the hex representation.
    pub fn as_hex(&self) -> &str {
        &self.hex
    }

    /// Return a short prefix (first 8 hex chars) for display.
    pub fn short(&self) -> &str {
        &self.hex[..8]
    }
}

impl<D: DomainTag> fmt::Debug for DeterministicId<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}…)", D::tag(), self.short())
    }
}

impl<D: DomainTag> fmt::Display for DeterministicId<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex)
    }
}

// ---------------------------------------------------------------------------
// Built-in domain tags
// ---------------------------------------------------------------------------

/// Domain tag for node IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeDomain;
impl DomainTag for NodeDomain {
    fn tag() -> &'static str {
        "node"
    }
}

/// Domain tag for edge IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeDomain;
impl DomainTag for EdgeDomain {
    fn tag() -> &'static str {
        "edge"
    }
}

/// Domain tag for graph IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphDomain;
impl DomainTag for GraphDomain {
    fn tag() -> &'static str {
        "graph"
    }
}

/// Domain tag for agent IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentDomain;
impl DomainTag for AgentDomain {
    fn tag() -> &'static str {
        "agent"
    }
}

/// Domain tag for blueprint IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlueprintDomain;
impl DomainTag for BlueprintDomain {
    fn tag() -> &'static str {
        "blueprint"
    }
}

/// Domain tag for task IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskDomain;
impl DomainTag for TaskDomain {
    fn tag() -> &'static str {
        "task"
    }
}

/// Convenience type aliases.
pub type NodeId = DeterministicId<NodeDomain>;
pub type EdgeId = DeterministicId<EdgeDomain>;
pub type GraphId = DeterministicId<GraphDomain>;
pub type AgentId = DeterministicId<AgentDomain>;
pub type BlueprintId = DeterministicId<BlueprintDomain>;
pub type TaskId = DeterministicId<TaskDomain>;

// ---------------------------------------------------------------------------
// Hex encoding helper (no external dependency)
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_id_is_deterministic() {
        let id1 = NodeId::from_content("hello");
        let id2 = NodeId::from_content("hello");
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_content_different_id() {
        let id1 = NodeId::from_content("hello");
        let id2 = NodeId::from_content("world");
        assert_ne!(id1, id2);
    }

    #[test]
    fn different_domain_different_id() {
        let node_id = NodeId::from_content("hello");
        let edge_id = EdgeId::from_content("hello");
        assert_ne!(node_id.as_hex(), edge_id.as_hex());
    }

    #[test]
    fn hex_roundtrip() {
        let id = NodeId::from_content("test");
        let hex = id.as_hex();
        let id2 = NodeId::from_hex(hex).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn invalid_hex_rejected() {
        assert!(NodeId::from_hex("not_valid_hex").is_err());
        assert!(NodeId::from_hex("abcd").is_err()); // too short
    }

    #[test]
    fn from_parts_is_deterministic() {
        let id1 = NodeId::from_parts(&[b"hello", b"world"]);
        let id2 = NodeId::from_parts(&[b"hello", b"world"]);
        assert_eq!(id1, id2);
    }

    #[test]
    fn short_returns_prefix() {
        let id = NodeId::from_content("test");
        assert_eq!(id.short().len(), 8);
    }
}
