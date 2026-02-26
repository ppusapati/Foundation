//! # Foundation Crate
//!
//! Core deterministic types, context, traits, transport, IDs, crypto, and errors
//! for the platform. All types are designed for deterministic execution –
//! no hidden randomness, no ambient I/O, no floating-point non-determinism.
//!
//! ## Modules
//!
//! - [`errors`] – Error types and result aliases
//! - [`primitives`] – Safe numeric wrappers (M2)
//! - [`id`] – Generic deterministic ID framework (M22)
//! - [`context`] – Deterministic execution context (M1)
//! - [`hash`] – SHA-256 hashing helpers (M8)
//! - [`serialize`] – Canonical JSON serialization (M9)
//! - [`time`] – Logical timestamps and clocks (M10)
//! - [`audit`] – Audit trail logging (M15)
//! - [`traits`] – Cross-system trait definitions (M7)
//! - [`types`] – Cross-system compound types (M3)
//! - [`transport`] – Transport types, router, and clients (M19-M21)

pub mod audit;
pub mod context;
pub mod errors;
pub mod hash;
pub mod id;
pub mod primitives;
pub mod serialize;
pub mod time;
pub mod traits;
pub mod transport;
pub mod types;

// Re-export key types at crate root for convenience.
pub use context::DeterministicContext;
pub use errors::{FoundationError, FoundationResult};
pub use id::{AgentId, BlueprintId, DeterministicId, DomainTag, EdgeId, GraphId, NodeId, TaskId};
pub use primitives::{BoundedFloat, NonNegativeFloat, Percentage, SafeFloat, SafeInteger};
pub use time::{LogicalClock, LogicalTimestamp, WallTimestamp};
pub use traits::{
    CanonicalSerializable, Diffable, Hashable, Identifiable, Mergeable, Snapshotable,
    Timestamped, Validatable,
};
pub use transport::{
    MessageId, MessagePayload, MessageQueue, MessageRouter, Priority, ResponseStatus,
    SystemAddress, TransportClient, TransportMessage,
};
pub use types::{
    EdgeData, GraphMetadata, NodeData, OperationResult, PropertyMap, PropertyValue, StatusCode,
    VersionInfo,
};
