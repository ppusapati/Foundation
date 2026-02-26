//! M3: Compound Types – cross-system data structures used throughout the platform.

use crate::errors::{FoundationError, FoundationResult};
use crate::id::{DeterministicId, EdgeDomain, GraphDomain, NodeDomain};
use crate::primitives::{SafeFloat, SafeInteger};
use crate::time::LogicalTimestamp;
use crate::traits::{Hashable, Identifiable, Validatable};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

// ---------------------------------------------------------------------------
// PropertyMap – a deterministic key-value store for entity metadata
// ---------------------------------------------------------------------------

/// An ordered key-value property map. Uses `BTreeMap` for deterministic iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyMap {
    inner: BTreeMap<String, PropertyValue>,
}

/// A value that can be stored in a `PropertyMap`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    Integer(SafeInteger),
    Float(SafeFloat),
    Text(String),
    Bool(bool),
    List(Vec<PropertyValue>),
}

impl PropertyMap {
    pub fn new() -> Self {
        PropertyMap {
            inner: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.inner.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<PropertyValue> {
        self.inner.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropertyValue)> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for PropertyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Hashable for PropertyMap {
    fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// NodeData – a generic node in the graph
// ---------------------------------------------------------------------------

/// A node entity with an ID, label, properties, and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: DeterministicId<NodeDomain>,
    pub label: String,
    pub properties: PropertyMap,
    pub created_at: LogicalTimestamp,
    pub updated_at: LogicalTimestamp,
}

impl NodeData {
    pub fn new(label: &str, properties: PropertyMap, ts: LogicalTimestamp) -> Self {
        let id_content = format!("{}:{}", label, serde_json::to_string(&properties).unwrap_or_default());
        NodeData {
            id: DeterministicId::from_content(&id_content),
            label: label.to_string(),
            properties,
            created_at: ts,
            updated_at: ts,
        }
    }
}

impl Identifiable for NodeData {
    type Domain = NodeDomain;
    fn id(&self) -> DeterministicId<NodeDomain> {
        self.id.clone()
    }
}

impl Hashable for NodeData {
    fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

impl Validatable for NodeData {
    fn validate(&self) -> FoundationResult<()> {
        if self.label.is_empty() {
            return Err(FoundationError::ValidationFailed(
                "node label must not be empty".into(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({}, label={})", self.id.short(), self.label)
    }
}

// ---------------------------------------------------------------------------
// EdgeData – a directed edge between two nodes
// ---------------------------------------------------------------------------

/// A directed edge connecting two nodes, with a relation type and properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: DeterministicId<EdgeDomain>,
    pub source: DeterministicId<NodeDomain>,
    pub target: DeterministicId<NodeDomain>,
    pub relation: String,
    pub weight: SafeFloat,
    pub properties: PropertyMap,
    pub created_at: LogicalTimestamp,
}

impl EdgeData {
    pub fn new(
        source: DeterministicId<NodeDomain>,
        target: DeterministicId<NodeDomain>,
        relation: &str,
        weight: SafeFloat,
        properties: PropertyMap,
        ts: LogicalTimestamp,
    ) -> Self {
        let id_content = format!(
            "{}:{}:{}",
            source.as_hex(),
            target.as_hex(),
            relation
        );
        EdgeData {
            id: DeterministicId::from_content(&id_content),
            source,
            target,
            relation: relation.to_string(),
            weight,
            properties,
            created_at: ts,
        }
    }
}

impl Identifiable for EdgeData {
    type Domain = EdgeDomain;
    fn id(&self) -> DeterministicId<EdgeDomain> {
        self.id.clone()
    }
}

impl Hashable for EdgeData {
    fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

impl Validatable for EdgeData {
    fn validate(&self) -> FoundationResult<()> {
        if self.relation.is_empty() {
            return Err(FoundationError::ValidationFailed(
                "edge relation must not be empty".into(),
            ));
        }
        if self.source == self.target {
            return Err(FoundationError::ValidationFailed(
                "self-loops are not allowed".into(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for EdgeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge({} --[{}]--> {})",
            self.source.short(),
            self.relation,
            self.target.short()
        )
    }
}

// ---------------------------------------------------------------------------
// GraphMetadata – metadata for a graph instance
// ---------------------------------------------------------------------------

/// Metadata describing a graph instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub id: DeterministicId<GraphDomain>,
    pub name: String,
    pub description: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub properties: PropertyMap,
    pub created_at: LogicalTimestamp,
    pub updated_at: LogicalTimestamp,
}

impl GraphMetadata {
    pub fn new(name: &str, description: &str, ts: LogicalTimestamp) -> Self {
        let id = DeterministicId::from_content(name);
        GraphMetadata {
            id,
            name: name.to_string(),
            description: description.to_string(),
            node_count: 0,
            edge_count: 0,
            properties: PropertyMap::new(),
            created_at: ts,
            updated_at: ts,
        }
    }
}

impl Identifiable for GraphMetadata {
    type Domain = GraphDomain;
    fn id(&self) -> DeterministicId<GraphDomain> {
        self.id.clone()
    }
}

impl Hashable for GraphMetadata {
    fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// VersionInfo – semantic version tracking
// ---------------------------------------------------------------------------

/// Semantic version information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl VersionInfo {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        VersionInfo {
            major,
            minor,
            patch,
        }
    }

    pub fn bump_major(&self) -> Self {
        VersionInfo {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    pub fn bump_minor(&self) -> Self {
        VersionInfo {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    pub fn bump_patch(&self) -> Self {
        VersionInfo {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// ---------------------------------------------------------------------------
// StatusCode – operation result status
// ---------------------------------------------------------------------------

/// A status code indicating the result of an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCode {
    Ok,
    Created,
    Updated,
    Deleted,
    NotFound,
    Conflict,
    ValidationError,
    InternalError,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::Ok => write!(f, "OK"),
            StatusCode::Created => write!(f, "CREATED"),
            StatusCode::Updated => write!(f, "UPDATED"),
            StatusCode::Deleted => write!(f, "DELETED"),
            StatusCode::NotFound => write!(f, "NOT_FOUND"),
            StatusCode::Conflict => write!(f, "CONFLICT"),
            StatusCode::ValidationError => write!(f, "VALIDATION_ERROR"),
            StatusCode::InternalError => write!(f, "INTERNAL_ERROR"),
        }
    }
}

// ---------------------------------------------------------------------------
// OperationResult – wraps a result with status and metadata
// ---------------------------------------------------------------------------

/// A result wrapper that includes status code and optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub status: StatusCode,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: LogicalTimestamp,
}

impl<T> OperationResult<T> {
    pub fn ok(data: T, ts: LogicalTimestamp) -> Self {
        OperationResult {
            status: StatusCode::Ok,
            data: Some(data),
            message: None,
            timestamp: ts,
        }
    }

    pub fn error(status: StatusCode, message: &str, ts: LogicalTimestamp) -> Self {
        OperationResult {
            status,
            data: None,
            message: Some(message.to_string()),
            timestamp: ts,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            StatusCode::Ok | StatusCode::Created | StatusCode::Updated | StatusCode::Deleted
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_map_basic() {
        let mut pm = PropertyMap::new();
        pm.set("name", PropertyValue::Text("test".into()));
        pm.set("count", PropertyValue::Integer(SafeInteger::new(42)));
        assert_eq!(pm.len(), 2);
        assert!(pm.contains_key("name"));
    }

    #[test]
    fn node_data_creation() {
        let node = NodeData::new("person", PropertyMap::new(), LogicalTimestamp::ZERO);
        assert_eq!(node.label, "person");
        assert!(node.validate().is_ok());
    }

    #[test]
    fn node_data_empty_label_invalid() {
        let node = NodeData::new("", PropertyMap::new(), LogicalTimestamp::ZERO);
        assert!(node.validate().is_err());
    }

    #[test]
    fn edge_data_creation() {
        let src = DeterministicId::from_content("a");
        let tgt = DeterministicId::from_content("b");
        let edge = EdgeData::new(
            src,
            tgt,
            "knows",
            SafeFloat::ONE,
            PropertyMap::new(),
            LogicalTimestamp::ZERO,
        );
        assert_eq!(edge.relation, "knows");
        assert!(edge.validate().is_ok());
    }

    #[test]
    fn edge_self_loop_invalid() {
        let id = DeterministicId::from_content("a");
        let edge = EdgeData::new(
            id.clone(),
            id,
            "self",
            SafeFloat::ONE,
            PropertyMap::new(),
            LogicalTimestamp::ZERO,
        );
        assert!(edge.validate().is_err());
    }

    #[test]
    fn version_info_bump() {
        let v = VersionInfo::new(1, 2, 3);
        assert_eq!(v.bump_patch(), VersionInfo::new(1, 2, 4));
        assert_eq!(v.bump_minor(), VersionInfo::new(1, 3, 0));
        assert_eq!(v.bump_major(), VersionInfo::new(2, 0, 0));
    }

    #[test]
    fn operation_result_ok() {
        let result = OperationResult::ok(42, LogicalTimestamp::ZERO);
        assert!(result.is_success());
        assert_eq!(result.data, Some(42));
    }

    #[test]
    fn operation_result_error() {
        let result: OperationResult<i32> =
            OperationResult::error(StatusCode::NotFound, "not found", LogicalTimestamp::ZERO);
        assert!(!result.is_success());
    }
}
