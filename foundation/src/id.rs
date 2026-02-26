//! M22: Generic Deterministic ID Framework.
//!
//! Provides a type-safe, domain-tagged, deterministic ID system.
//! IDs are SHA-256 hashes of `(domain_tag || canonical_bytes)`.
//! Stored as raw `[u8; 32]` for zero-allocation comparisons and hashing.

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

/// A deterministic, domain-tagged identifier stored as raw SHA-256 bytes.
///
/// Using `[u8; 32]` instead of `String` provides:
/// - Zero heap allocation per ID (Copy-able)
/// - Faster comparisons (32-byte memcmp)
/// - Smaller memory footprint (32 bytes vs 24 + 64 heap bytes)
#[derive(Clone, Copy)]
pub struct DeterministicId<D: DomainTag> {
    bytes: [u8; 32],
    _marker: PhantomData<D>,
}

// Manual trait impls to avoid requiring D: PartialEq/Eq/Hash/Ord,
// since PhantomData<D> doesn't participate in comparison or hashing.

impl<D: DomainTag> PartialEq for DeterministicId<D> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<D: DomainTag> Eq for DeterministicId<D> {}

impl<D: DomainTag> std::hash::Hash for DeterministicId<D> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<D: DomainTag> PartialOrd for DeterministicId<D> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<D: DomainTag> Ord for DeterministicId<D> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<D: DomainTag> DeterministicId<D> {
    /// Generate an ID by hashing `domain_tag || content_bytes`.
    #[inline]
    pub fn generate(content_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(D::tag().as_bytes());
        hasher.update(content_bytes);
        let hash = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        DeterministicId {
            bytes,
            _marker: PhantomData,
        }
    }

    /// Generate an ID from a string representation of the content.
    #[inline]
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
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        DeterministicId {
            bytes,
            _marker: PhantomData,
        }
    }

    /// Create a `DeterministicId` from an existing hex string.
    /// Validates that the string is a 64-character hex string.
    pub fn from_hex(hex: &str) -> FoundationResult<Self> {
        if hex.len() != 64 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(FoundationError::IdError(
                "ID must be a 64-character hex string".into(),
            ));
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            bytes[i] = hex_byte(chunk[0]) << 4 | hex_byte(chunk[1]);
        }
        Ok(DeterministicId {
            bytes,
            _marker: PhantomData,
        })
    }

    /// Return the raw bytes of the hash.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Return the hex representation as a new String.
    #[inline]
    pub fn as_hex(&self) -> String {
        hex_encode(&self.bytes)
    }

    /// Return a short prefix (first 8 hex chars) for display.
    #[inline]
    pub fn short(&self) -> String {
        hex_encode(&self.bytes[..4])
    }
}

/// Custom Serialize: write as hex string for JSON compatibility.
impl<D: DomainTag> Serialize for DeterministicId<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_hex())
    }
}

/// Custom Deserialize: read from hex string.
impl<'de, D: DomainTag> Deserialize<'de> for DeterministicId<D> {
    fn deserialize<De: serde::Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        let hex = String::deserialize(deserializer)?;
        Self::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

impl<D: DomainTag> fmt::Debug for DeterministicId<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}…)", D::tag(), self.short())
    }
}

impl<D: DomainTag> fmt::Display for DeterministicId<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in &self.bytes {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
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
// Hex encoding/decoding helpers (no external dependency)
// ---------------------------------------------------------------------------

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

#[inline]
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

#[inline]
fn hex_byte(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
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
        let id2 = NodeId::from_hex(&hex).unwrap();
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

    #[test]
    fn id_is_copy() {
        let id = NodeId::from_content("copy_test");
        let id2 = id; // Copy, not move
        assert_eq!(id, id2);
    }

    #[test]
    fn id_is_orderable() {
        let id1 = NodeId::from_content("aaa");
        let id2 = NodeId::from_content("bbb");
        // Just verify Ord works (deterministic ordering)
        let _ = id1.cmp(&id2);
    }

    #[test]
    fn serde_roundtrip() {
        let id = NodeId::from_content("serde_test");
        let json = serde_json::to_string(&id).unwrap();
        let id2: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, id2);
    }
}
