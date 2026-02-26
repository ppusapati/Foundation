//! # Utils Crate
//!
//! Utility modules for the platform: math, linear algebra, graph algorithms,
//! validation, audit, and determinism helpers.
//!
//! ## Modules
//!
//! - [`graph_types`] – Graph-domain compound types (M4)
//! - [`agent_types`] – Agent-domain compound types (M5)
//! - [`blueprint_types`] – Blueprint-domain compound types (M6)
//! - [`math`] – Deterministic math helpers (M11)
//! - [`linalg`] – Linear algebra: vectors and matrices (M12)
//! - [`validation`] – Validation combinators (M13)
//! - [`determinism`] – Determinism enforcement utilities (M14)
//! - [`graph`] – Graph algorithm helpers (M17)

pub mod agent_types;
pub mod blueprint_types;
pub mod determinism;
pub mod graph;
pub mod graph_types;
pub mod linalg;
pub mod math;
pub mod validation;

// Re-export key types at crate root.
pub use agent_types::{AgentMetrics, AgentProfile, AgentState, TaskDescriptor, TaskPriority, TaskStatus};
pub use blueprint_types::{BlueprintDef, BlueprintInstance, BlueprintStatus, BlueprintStep, ParameterDef, ParameterType};
pub use graph_types::{AdjacencyList, GraphPath, GraphQuery, GraphStats, SubgraphView};
pub use linalg::{Matrix, Vector};
pub use validation::ValidationCollector;
