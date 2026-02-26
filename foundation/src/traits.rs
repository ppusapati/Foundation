//! M7: Cross-System Traits – shared trait definitions that multiple systems implement.

use crate::context::DeterministicContext;
use crate::errors::FoundationResult;
use crate::id::DomainTag;
use crate::time::LogicalTimestamp;
use serde::{de::DeserializeOwned, Serialize};

/// A type that can produce a deterministic hash of its contents.
pub trait Hashable {
    /// Return the canonical bytes used for hashing.
    fn canonical_bytes(&self) -> Vec<u8>;

    /// Return the SHA-256 hex hash.
    fn content_hash(&self) -> String {
        crate::hash::sha256_hex(&self.canonical_bytes())
    }
}

/// A type that can be validated for internal consistency.
pub trait Validatable {
    /// Check internal invariants. Returns `Ok(())` if valid.
    fn validate(&self) -> FoundationResult<()>;
}

/// A type that can be deterministically merged with another of the same type.
pub trait Mergeable: Sized {
    /// Merge `self` with `other`, producing a new value.
    fn merge(&self, other: &Self, ctx: &mut DeterministicContext) -> FoundationResult<Self>;
}

/// A type that has a deterministic identifier.
pub trait Identifiable {
    /// The domain tag type for this entity's ID.
    type Domain: DomainTag;

    /// Return the deterministic ID of this entity.
    fn id(&self) -> crate::id::DeterministicId<Self::Domain>;
}

/// A type that tracks when it was last modified (logical time).
pub trait Timestamped {
    /// Return the logical timestamp of the last modification.
    fn last_modified(&self) -> LogicalTimestamp;
}

/// A type that can be serialized to and from canonical JSON.
pub trait CanonicalSerializable: Serialize + DeserializeOwned {
    /// Serialize to canonical JSON.
    fn to_canonical_json(&self) -> FoundationResult<String> {
        crate::serialize::to_canonical_json(self)
    }

    /// Deserialize from JSON string.
    fn from_canonical_json(json: &str) -> FoundationResult<Self> {
        crate::serialize::from_json_str(json)
    }
}

/// A type that supports deterministic diff computation.
pub trait Diffable: Sized {
    /// The type representing the difference.
    type Delta;

    /// Compute the difference between `self` and `other`.
    fn diff(&self, other: &Self) -> FoundationResult<Self::Delta>;

    /// Apply a delta to produce a new value.
    fn apply_delta(&self, delta: &Self::Delta) -> FoundationResult<Self>;
}

/// A type that can be snapshotted and restored.
pub trait Snapshotable: Sized + Clone {
    /// Create a snapshot (deep clone with metadata).
    fn snapshot(&self) -> Self {
        self.clone()
    }
}

/// A deterministic step that transforms state within a context.
pub trait DeterministicStep {
    /// Input type.
    type Input;
    /// Output type.
    type Output;

    /// Execute the step within the given context.
    fn execute(
        &self,
        input: &Self::Input,
        ctx: &mut DeterministicContext,
    ) -> FoundationResult<Self::Output>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{DeterministicId, NodeDomain};

    #[derive(Clone)]
    struct TestNode {
        name: String,
    }

    impl Hashable for TestNode {
        fn canonical_bytes(&self) -> Vec<u8> {
            self.name.as_bytes().to_vec()
        }
    }

    impl Validatable for TestNode {
        fn validate(&self) -> FoundationResult<()> {
            if self.name.is_empty() {
                Err(crate::errors::FoundationError::ValidationFailed(
                    "name is empty".into(),
                ))
            } else {
                Ok(())
            }
        }
    }

    impl Identifiable for TestNode {
        type Domain = NodeDomain;
        fn id(&self) -> DeterministicId<NodeDomain> {
            DeterministicId::from_content(&self.name)
        }
    }

    #[test]
    fn hashable_impl() {
        let node = TestNode {
            name: "test".into(),
        };
        let hash = node.content_hash();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn validatable_impl() {
        let good = TestNode {
            name: "ok".into(),
        };
        assert!(good.validate().is_ok());

        let bad = TestNode {
            name: "".into(),
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn identifiable_impl() {
        let node = TestNode {
            name: "test".into(),
        };
        let id = node.id();
        assert_eq!(id.as_hex().len(), 64);
    }
}
