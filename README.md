# Foundation

A Rust platform for **deterministic execution** — every operation produces the same output given the same input, with no hidden randomness, no ambient I/O, and no floating-point non-determinism.

Built as a workspace of two crates:

- **`foundation`** — Core types, deterministic context, traits, transport, and IDs
- **`utils`** — Math, linear algebra, graph algorithms, validation, and agent/blueprint domain types

## Why Determinism?

Systems that must be auditable, reproducible, or formally verifiable need execution guarantees that typical runtimes don't provide. Foundation enforces determinism at the type level:

- **Checked arithmetic everywhere.** `SafeInteger` and `SafeFloat` wrap `i64`/`f64` with overflow-checked operations — no silent wrapping, no NaN propagation.
- **Content-addressed IDs.** `DeterministicId<D>` produces SHA-256 IDs from `(domain_tag || content)`, stored as raw `[u8; 32]` for zero-allocation comparisons. Domain tags (Node, Edge, Graph, Agent, Task, Blueprint) make ID collisions across domains impossible at the type level.
- **Logical time, not wall clocks.** `LogicalTimestamp` and `LogicalClock` provide Lamport-style causal ordering. Wall-clock timestamps exist for human-readable logs only.
- **Canonical serialization.** All JSON output is deterministic — same struct always serializes to the same bytes.
- **Execution context.** `DeterministicContext` carries the logical clock, a reproducible seed, a frozen config snapshot, and an execution trace. It's passed by reference to every operation.

## Architecture

### `foundation` crate (Core)

| Module | Spec | Description |
|---|---|---|
| `context` | M1 | `DeterministicContext` — immutable execution context with logical clock, seed, config, and trace |
| `primitives` | M2 | `SafeInteger`, `SafeFloat`, `Percentage`, `BoundedFloat`, `NonNegativeFloat` |
| `types` | M3 | Cross-system compound types: `NodeData`, `EdgeData`, `GraphMetadata`, `PropertyMap`, `VersionInfo` |
| `traits` | M7 | Shared traits: `Hashable`, `Validatable`, `Mergeable`, `Identifiable`, `Timestamped`, `Diffable`, `Snapshotable` |
| `hash` | M8 | SHA-256 hashing with lookup-table hex encoding |
| `serialize` | M9 | Canonical JSON serialization/deserialization |
| `time` | M10 | `LogicalTimestamp`, `LogicalClock`, `WallTimestamp` |
| `audit` | M15 | Structured audit trail with severity levels and event logging |
| `transport` | M19-21 | Message types, routing, and client abstractions for inter-system communication |
| `id` | M22 | Generic `DeterministicId<D>` framework with domain-tagged SHA-256 IDs |
| `errors` | — | `FoundationError` enum and `FoundationResult<T>` alias |

### `utils` crate (Utilities)

| Module | Spec | Description |
|---|---|---|
| `graph_types` | M4 | `AdjacencyList` (with CSR storage), `GraphPath`, `SubgraphView`, `GraphQuery`, `GraphStats` |
| `agent_types` | M5 | `AgentProfile`, `AgentState`, `TaskDescriptor`, `TaskPriority`, `AgentMetrics` |
| `blueprint_types` | M6 | `BlueprintDef`, `BlueprintStep`, `ParameterDef`, `BlueprintInstance` with lifecycle management |
| `math` | M11 | Deterministic math: GCD, LCM, clamping, linear interpolation, moving average, weighted sum |
| `linalg` | M12 | Dense `Vector` and `Matrix` with safe arithmetic (dot product, norms, cosine similarity, matrix multiply) |
| `validation` | M13 | Validation combinators: string/numeric constraints, uniqueness checks, `ValidationCollector` |
| `determinism` | M14 | Deterministic sort, shuffle (seeded PRNG), selection, hash-based tiebreaking, `DeterminismGuard` |
| `graph` | M17 | Graph algorithms: BFS, DFS, Dijkstra, topological sort, cycle detection, connected components, PageRank |

## Performance

The codebase is optimized for zero unnecessary allocation in hot paths:

- **Graph algorithms** use index-based internals (`Vec<bool>` visited, `Vec<f64>` distances, `usize` heap entries) with conversion to `String` IDs only at the return boundary.
- **CSR (Compressed Sparse Row)** storage compacts adjacency lists into flat arrays with pre-sorted edges, giving algorithms contiguous cache-friendly slices with zero per-visit allocation.
- **`DeterministicId`** is `Copy` (32-byte stack value) — no heap allocation for ID creation, comparison, or hashing.
- **Matrix multiplication** uses i-k-j loop order for cache-friendly row-major access.
- **Hex encoding** uses a lookup table instead of per-byte `format!()`.

## Quick Start

```bash
# Build
cargo build --workspace

# Run all tests (135 tests)
cargo test --workspace

# Check for warnings
cargo clippy --workspace -- -D warnings
```

### Example: Deterministic Graph Traversal

```rust
use utils::graph_types::AdjacencyList;
use utils::graph::{bfs, dijkstra};
use foundation::primitives::SafeFloat;

// Build a graph
let mut graph = AdjacencyList::new();
graph.add_edge("a", "b", SafeFloat::ONE);
graph.add_edge("a", "c", SafeFloat::new(2.0).unwrap());
graph.add_edge("b", "d", SafeFloat::ONE);

// Compact into CSR for deterministic, zero-allocation traversal
graph.compact();

// BFS always returns the same order
let order = bfs(&graph, "a").unwrap();
// order: ["a", "b", "c", "d"]

// Dijkstra's shortest paths
let (distances, predecessors) = dijkstra(&graph, "a").unwrap();
```

### Example: Safe Arithmetic

```rust
use foundation::primitives::{SafeFloat, SafeInteger};

let a = SafeInteger::new(i64::MAX);
let b = SafeInteger::ONE;
assert!(a.checked_add(b).is_err()); // overflow caught, not silent

let x = SafeFloat::new(3.0).unwrap();
let y = SafeFloat::new(4.0).unwrap();
let sum = x.checked_add(y).unwrap(); // 7.0, verified finite
```

### Example: Deterministic IDs

```rust
use foundation::id::{DeterministicId, NodeDomain};

// Same content always produces the same ID
let id1: DeterministicId<NodeDomain> = DeterministicId::from_content("user:alice");
let id2: DeterministicId<NodeDomain> = DeterministicId::from_content("user:alice");
assert_eq!(id1, id2);

// Different domains produce different IDs even for the same content
let node_id: DeterministicId<NodeDomain> = DeterministicId::from_content("x");
// node_id != edge_id with same content, enforced at the type level
```

## Requirements

- Rust 1.70+
- No runtime dependencies beyond `serde`, `serde_json`, `sha2`, `chrono`, and `uuid`

## Design Principles

1. **No hidden state.** Every input to a computation is explicit. No global singletons, no thread-local storage, no ambient authority.
2. **Fail loud.** Overflow, NaN, out-of-range — all are errors, never silent corruption. `Result<T, FoundationError>` throughout.
3. **Types prevent mistakes.** `DeterministicId<NodeDomain>` can't be used where `DeterministicId<EdgeDomain>` is expected. `SafeFloat` can't silently become NaN.
4. **Deterministic ordering.** BTreeMap over HashMap for iteration. Logical timestamps over wall clocks. Sorted edge lists for traversal.
5. **Audit everything.** The context carries an execution trace. The audit module provides structured event logging with logical and wall timestamps.
