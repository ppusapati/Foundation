# Foundation Help Guide

## What Is Foundation?

Foundation is a Rust library for building systems where **every computation must be reproducible, auditable, and verifiable**. It enforces determinism at the type level — silent overflows, NaN propagation, non-deterministic iteration order, and ambient I/O are all structurally prevented.

It is a workspace of two crates:

- **`foundation`** — Core primitives, execution context, IDs, time, hashing, serialization, audit, transport, and traits
- **`utils`** — Math, linear algebra, graph algorithms, validation, determinism enforcement, and domain types (agents, blueprints)

---

## Where Is This Useful?

### 1. Reproducible Multi-Agent Systems

Foundation provides `AgentProfile`, `TaskDescriptor`, and `AgentMetrics` for modeling agent-based systems where every decision must be traceable and replayed. The `DeterministicContext` carries a seed, logical clock, and execution trace, so any agent operation can be re-executed with identical results.

**Use case:** AI agent orchestration platforms, autonomous planning systems, simulation frameworks where agent behavior must be auditable.

### 2. Knowledge Graphs and Graph Analytics

The `AdjacencyList` with CSR storage, combined with BFS, DFS, Dijkstra, topological sort, cycle detection, connected components, and PageRank algorithms, provides a high-performance graph engine. All traversals are deterministic — the same graph always produces the same output.

**Use case:** Knowledge graph construction, dependency resolution, network analysis, workflow DAG execution.

### 3. Scientific and Financial Computing

`SafeInteger` and `SafeFloat` catch every overflow, division-by-zero, and NaN before they corrupt a computation. The `linalg` module provides vectors and matrices with checked arithmetic throughout. The `math` module includes statistical functions (mean, variance, std deviation) that never silently produce garbage.

**Use case:** Financial modeling, scientific simulation, risk analysis — anywhere a silent arithmetic error could cause real damage.

### 4. Auditable Event Systems

The `AuditLog` and `AuditEvent` types provide structured, append-only audit trails with both logical timestamps (for causal ordering) and wall-clock timestamps (for human readability). The transport layer (`TransportMessage`, `MessageRouter`, `MessageQueue`) provides deterministic inter-system messaging with priority-based delivery.

**Use case:** Compliance systems, regulated industries, event sourcing architectures, distributed system debugging.

### 5. Blueprint/Template Engines

`BlueprintDef` models reusable operation templates with typed parameters, step dependencies, lifecycle management (Draft → Active → Deprecated → Archived), and instantiation with validation. Step dependency graphs are validated for correctness.

**Use case:** CI/CD pipeline definitions, workflow templates, infrastructure-as-code, configurable operation sequences.

### 6. Deterministic Testing and Verification

The `determinism` module provides tools for verifying that operations produce identical results: `DeterminismGuard` (hash-before/hash-after verification), `verify_deterministic_execution` (run-twice-and-compare), deterministic shuffle/select (seeded PRNG), and hash-based tiebreaking.

**Use case:** Property-based testing infrastructure, formal verification tooling, deterministic replay systems.

---

## How to Use It

### Installation

Add Foundation to your project:

```toml
# Cargo.toml
[dependencies]
foundation = { path = "path/to/Foundation/foundation" }
utils = { path = "path/to/Foundation/utils" }
```

### Building and Testing

```bash
# Build the entire workspace
cargo build --workspace

# Run all 135 tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Lint check (zero warnings policy)
cargo clippy --workspace -- -D warnings
```

### Using Safe Arithmetic

Every arithmetic operation returns a `Result`. No silent corruption.

```rust
use foundation::primitives::{SafeInteger, SafeFloat, Percentage};

// Integer arithmetic — overflow is an error, not a wrap
let a = SafeInteger::new(100);
let b = SafeInteger::new(200);
let sum = a.checked_add(b).unwrap();       // Ok(300)
let product = a.checked_mul(b).unwrap();   // Ok(20000)

let big = SafeInteger::MAX;
assert!(big.checked_add(SafeInteger::ONE).is_err()); // caught

// Float arithmetic — NaN and Infinity are rejected at construction
let x = SafeFloat::new(3.14).unwrap();
let y = SafeFloat::new(2.0).unwrap();
let result = x.checked_mul(y).unwrap();    // Ok(6.28)

assert!(SafeFloat::new(f64::NAN).is_err());       // rejected
assert!(SafeFloat::new(f64::INFINITY).is_err());   // rejected

// Constrained types
let pct = Percentage::new(85.0).unwrap();  // must be [0, 100]
assert!(Percentage::new(101.0).is_err());  // rejected
```

### Using Deterministic IDs

IDs are content-addressed (SHA-256) and domain-tagged. Same input always produces the same ID.

```rust
use foundation::id::{DeterministicId, NodeDomain, EdgeDomain};

// Create IDs from content — deterministic and reproducible
let id1: DeterministicId<NodeDomain> = DeterministicId::from_content("user:alice");
let id2: DeterministicId<NodeDomain> = DeterministicId::from_content("user:alice");
assert_eq!(id1, id2); // same content → same ID, always

// IDs are Copy — no heap allocation, 32 bytes on the stack
let id_copy = id1; // no clone needed

// Domain safety — NodeDomain ID cannot be used where EdgeDomain is expected
// let edge_id: DeterministicId<EdgeDomain> = id1; // compile error!

// Display as hex
println!("{}", id1);       // full 64-char hex
println!("{}", id1.short()); // first 16 chars
```

### Using the Execution Context

The context is the core of deterministic execution. It carries the clock, seed, config, and trace.

```rust
use foundation::context::DeterministicContext;
use std::collections::BTreeMap;

// Create a context with a seed — same seed = same execution
let mut ctx = DeterministicContext::new(42);

// Advance the logical clock and record what happened
let ts = ctx.tick("process_order", "order-12345").unwrap();
println!("Operation happened at logical time: {}", ts); // T1

// Configuration is frozen at creation — no ambient state
let mut config = BTreeMap::new();
config.insert("max_retries".into(), "3".into());
let ctx = DeterministicContext::with_config(42, config);
assert_eq!(ctx.get_config("max_retries"), Some(&"3".to_string()));

// Derive child contexts for parallel work
let child = ctx.derive_child(1).unwrap();
assert_eq!(child.seed(), 43); // deterministic child seed

// Inspect the execution trace for auditing
for entry in ctx.trace() {
    println!("{}: {} — {}", entry.timestamp, entry.operation, entry.detail);
}
```

### Using Graph Algorithms

Build a graph, compact it, then run algorithms. All results are deterministic.

```rust
use utils::graph_types::AdjacencyList;
use utils::graph::{bfs, dfs, dijkstra, topological_sort, has_cycle, connected_components};
use foundation::primitives::SafeFloat;

// Build the graph
let mut graph = AdjacencyList::new();
graph.add_edge("a", "b", SafeFloat::ONE);
graph.add_edge("a", "c", SafeFloat::new(2.0).unwrap());
graph.add_edge("b", "d", SafeFloat::ONE);
graph.add_edge("c", "d", SafeFloat::new(3.0).unwrap());

// Compact into CSR format — call once after all edges are added
graph.compact();

// BFS traversal — deterministic order guaranteed
let order = bfs(&graph, "a").unwrap();
// Always: ["a", "b", "c", "d"]

// DFS traversal
let order = dfs(&graph, "a").unwrap();

// Shortest paths
let (distances, predecessors) = dijkstra(&graph, "a").unwrap();
// distances: {"a": 0.0, "b": 1.0, "c": 2.0, "d": 2.0}

// Topological ordering (fails if graph has a cycle)
let sorted = topological_sort(&graph).unwrap();

// Cycle detection
assert!(!has_cycle(&graph));

// Connected components
let components = connected_components(&graph);
```

### Using Linear Algebra

Dense vectors and matrices with checked arithmetic throughout.

```rust
use utils::linalg::{Vector, Matrix};
use foundation::primitives::SafeFloat;

// Vector operations
let a = Vector::from_f64(&[1.0, 2.0, 3.0]).unwrap();
let b = Vector::from_f64(&[4.0, 5.0, 6.0]).unwrap();

let sum = a.add(&b).unwrap();         // [5, 7, 9]
let dot = a.dot(&b).unwrap();         // 32.0
let norm = a.norm().unwrap();          // sqrt(14)
let unit = a.normalize().unwrap();     // unit vector
let sim = a.cosine_similarity(&b).unwrap(); // cosine similarity

// Matrix operations
let m = Matrix::identity(3);
let v = Vector::from_f64(&[1.0, 2.0, 3.0]).unwrap();
let result = m.mul_vec(&v).unwrap();   // [1, 2, 3] (identity)

let a = Matrix::zeros(2, 3);
let b = Matrix::zeros(3, 4);
let product = a.mul(&b).unwrap();      // 2x4 matrix
let transposed = product.transpose();  // 4x2 matrix
let trace = Matrix::identity(3).trace().unwrap(); // 3.0
```

### Using the Transport Layer

Deterministic inter-system messaging with routing and priority queues.

```rust
use foundation::transport::*;
use foundation::time::LogicalTimestamp;

// Create a request message
let msg = TransportMessage::request(
    MessageId::generate(42, 1),
    SystemAddress::AgentSystem,
    SystemAddress::GraphSystem,
    "create_node",
    serde_json::json!({"label": "person", "name": "Alice"}),
    LogicalTimestamp::new(1),
).with_priority(Priority::High);

// Set up a router
let mut router = MessageRouter::new();
router.register(
    SystemAddress::GraphSystem,
    Some("create_node".to_string()),
    "create_handler",
    Box::new(|msg| {
        Ok(TransportMessage::response(
            MessageId::generate(42, 2),
            msg,
            ResponseStatus::Success,
            serde_json::json!({"created": true}),
            LogicalTimestamp::new(2),
        ))
    }),
);

// Route the message
let response = router.route(&msg).unwrap();
```

### Using the Audit Trail

Structured, append-only audit logging with filtering.

```rust
use foundation::audit::{AuditEvent, AuditLevel, AuditLog};
use foundation::time::LogicalTimestamp;

let mut log = AuditLog::new();

log.record(
    AuditEvent::new(
        LogicalTimestamp::new(1),
        AuditLevel::Info,
        "order_service",
        "create_order",
        "Order ORD-123 created",
    )
    .with_context_id("ctx-001"),
);

log.record(
    AuditEvent::new(
        LogicalTimestamp::new(2),
        AuditLevel::Error,
        "payment_service",
        "charge",
        "Payment declined for ORD-123",
    )
    .with_context_id("ctx-001"),
);

// Filter by severity
let errors = log.events_at_level(AuditLevel::Error);     // 1 event
let important = log.events_at_min_level(AuditLevel::Info); // 2 events

// Filter by context (trace all operations for a request)
let ctx_events = log.events_for_context("ctx-001");       // 2 events
```

### Using Validation

Composable validation rules that collect multiple errors.

```rust
use utils::validation::*;

// Individual validators
non_empty_string("Alice", "name").unwrap();    // Ok
non_empty_string("", "name").unwrap_err();     // error

min_length("hello", 3, "greeting").unwrap();   // Ok
max_length("hi", 10, "greeting").unwrap();     // Ok

// Collect multiple validation errors at once
let mut collector = ValidationCollector::new();
collector.check(non_empty_string("", "name"));
collector.check(min_length("ab", 5, "code"));
let result = collector.finish(); // Err with all failures listed
```

### Using Blueprints

Define reusable operation templates with parameters and step dependencies.

```rust
use utils::blueprint_types::*;
use foundation::time::LogicalTimestamp;
use std::collections::BTreeMap;

// Define a blueprint
let mut bp = BlueprintDef::new("deploy-service", "Deploy a microservice", LogicalTimestamp::ZERO);

bp.add_parameter(
    ParameterDef::new("service_name", ParameterType::String, true)
        .with_description("Name of the service to deploy"),
);
bp.add_parameter(
    ParameterDef::new("replicas", ParameterType::Integer, false)
        .with_default("3"),
);

bp.add_step(BlueprintStep::new("build", "docker_build"));
bp.add_step(BlueprintStep::new("test", "run_tests").with_dependency("build"));
bp.add_step(BlueprintStep::new("deploy", "k8s_apply").with_dependency("test"));

// Validate step dependencies
bp.validate_steps().unwrap(); // Ok — all deps exist

// Activate the blueprint
bp.activate(LogicalTimestamp::new(1)).unwrap();

// Instantiate with concrete values
let mut params = BTreeMap::new();
params.insert("service_name".into(), "auth-api".into());
let instance = BlueprintInstance::new(&bp, params, LogicalTimestamp::new(2)).unwrap();
```

### Using Agent Types

Model agents with state machines and task assignment.

```rust
use utils::agent_types::*;
use foundation::time::LogicalTimestamp;

// Create an agent profile
let mut agent = AgentProfile::new("planner-1", "planner", LogicalTimestamp::ZERO);
agent.add_capability("code_review");
agent.add_capability("planning");

// Valid state transitions are enforced
agent.transition_to(AgentState::Planning, LogicalTimestamp::new(1)).unwrap();
agent.transition_to(AgentState::Executing, LogicalTimestamp::new(2)).unwrap();
// agent.transition_to(AgentState::Idle, _) would fail — Executing → Idle is invalid

// Create and assign tasks
let mut task = TaskDescriptor::new(
    "review-pr-42",
    "Review pull request #42",
    TaskPriority::High,
    LogicalTimestamp::new(3),
);
task.assign_to(agent.id, LogicalTimestamp::new(4));
task.update_status(TaskStatus::InProgress, LogicalTimestamp::new(5));
```

---

## Constraints and Limitations

### Arithmetic Constraints

| Constraint | Type | Behavior |
|---|---|---|
| Integer overflow | `SafeInteger` | Returns `Err(Overflow)` — never wraps silently |
| Integer division by zero | `SafeInteger` | Returns `Err(Overflow)` |
| Integer remainder by zero | `SafeInteger` | Returns `Err(Overflow)` |
| Negation of `i64::MIN` | `SafeInteger` | Returns `Err(Overflow)` — no representable positive value |
| `abs(i64::MIN)` | `SafeInteger` | Returns `Err(Overflow)` |
| NaN | `SafeFloat` | Rejected at construction — `new()` returns `Err(OutOfRange)` |
| Positive infinity | `SafeFloat` | Rejected at construction |
| Negative infinity | `SafeFloat` | Rejected at construction |
| Float division by zero | `SafeFloat` | Returns `Err(Overflow)` |
| Sqrt of negative number | `SafeFloat` | Returns `Err(OutOfRange)` |
| Result producing NaN/Inf | `SafeFloat` | `checked_add/sub/mul/div` return `Err` if result is non-finite |

### Numeric Range Constraints

| Type | Valid Range |
|---|---|
| `SafeInteger` | `i64::MIN` to `i64::MAX` (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807) |
| `SafeFloat` | Any finite `f64` (no NaN, no ±Infinity) |
| `Percentage` | `0.0` to `100.0` inclusive, must be finite |
| `NonNegativeFloat` | `0.0` to `f64::MAX`, must be finite |
| `BoundedFloat` | User-defined `[min, max]`, all values must be finite, `min ≤ max` required |

### Float Comparison Constraint

`SafeFloat` uses `PartialEq` based on exact `f64` equality (`==`), not approximate equality. For approximate comparison, use `.approx_eq()` (epsilon = `1e-10`) or `.approx_eq_eps()` with a custom epsilon. `SafeFloat` does **not** implement `Eq`, `Ord`, or `Hash` because floating-point ordering is not total.

### ID Constraints

| Constraint | Detail |
|---|---|
| Domain safety | `DeterministicId<NodeDomain>` and `DeterministicId<EdgeDomain>` are different types — cannot be mixed at compile time |
| Content-addressed | IDs are SHA-256 hashes of `(domain_tag \|\| content)` — same content in different domains produces different IDs |
| Fixed size | 32 bytes (`[u8; 32]`) — always exactly 64 hex characters when displayed |
| Immutable | IDs are `Copy` and cannot be modified after creation |
| Hash algorithm | SHA-256 only — not configurable |

### Logical Time Constraints

| Constraint | Detail |
|---|---|
| Monotonic only | `LogicalTimestamp` values are `u64` and can only increase |
| Overflow | `increment()` and `merge()` return `Err` if `u64::MAX` is reached |
| No wall-clock ordering | Logical timestamps express causal order, not real time — two events at `T5` and `T10` are not necessarily 5 time-units apart |
| Merge semantics | `merge(a, b) = max(a, b) + 1` (Lamport clock) |

### Graph Constraints

| Constraint | Detail |
|---|---|
| Two-phase lifecycle | Nodes and edges can only be added before `compact()`. Adding after compact panics |
| Directed edges only | `add_edge("a", "b", w)` adds `a → b`, not `b → a`. For undirected, add both directions |
| Non-negative weights | Edge weights are `SafeFloat` (must be finite). Dijkstra requires non-negative weights |
| Deterministic traversal | Requires calling `compact()` after construction. Without it, traversal order depends on insertion order |
| String node IDs | Nodes are identified by `&str` at the API boundary. Internally mapped to `usize` indices |
| No edge removal | Edges cannot be removed after addition. Build a new graph instead |
| No node removal | Nodes cannot be removed after addition |
| Dijkstra limitation | Returns `f64::MAX` for unreachable nodes, not an error |
| Topological sort | Fails with `Err` if the graph contains a cycle |

### Linear Algebra Constraints

| Constraint | Detail |
|---|---|
| Dimension matching | Vector addition, subtraction, and dot product require equal dimensions — mismatches return `Err` |
| Matrix multiplication | Requires `A.cols == B.rows` — incompatible dimensions return `Err` |
| Matrix-vector multiply | Requires `matrix.cols == vector.dim()` |
| Trace | Only defined for square matrices — non-square returns `Err` |
| Zero vector normalization | `normalize()` on a zero vector returns `Err` |
| Cosine similarity | Undefined for zero vectors — returns `Err` |
| Dense storage only | Matrices are stored as flat `Vec<SafeFloat>` in row-major order — no sparse matrix support |
| Every element is checked | All arithmetic goes through `SafeFloat`, so any operation producing NaN/Infinity fails |

### Blueprint Constraints

| Constraint | Detail |
|---|---|
| Lifecycle transitions | Draft → Active → Deprecated → Archived (forward only). Skipping states returns `Err` |
| Step dependency validation | `validate_steps()` checks that all `depends_on` references point to existing step names |
| Parameter uniqueness | `validate_parameters()` rejects duplicate parameter names |
| Required parameters | `BlueprintInstance::new()` fails if a required parameter has no value and no default |
| Instance IDs | Generated from `blueprint_id.short() + "-" + timestamp` — deterministic given inputs |

### Agent Constraints

| Constraint | Detail |
|---|---|
| State machine | Valid transitions: Idle→Planning, Planning→Executing, Executing→Waiting/Completed/Failed, Waiting→Executing, any→Suspended, Suspended→Idle, Failed→Idle, Completed→Idle. All others return `Err` |
| Capability deduplication | `add_capability()` is idempotent — adding the same capability twice has no effect |
| Task blocking | `is_blocked()` returns `true` if any dependency ID is not in the completed list |
| Agent ID | Generated from `"name:role"` — same name+role always produces the same agent ID |
| Task ID | Generated from `"name:timestamp"` — same name at the same logical time produces the same task ID |

### Transport Constraints

| Constraint | Detail |
|---|---|
| Router matching | Routes match on destination + operation pattern. First match wins. No match → `Err(TransportError)` and message added to undelivered queue |
| Message ordering | `MessageQueue` is FIFO. `drain_by_priority()` sorts by priority (Critical > High > Normal > Low) |
| No persistence | Transport is in-memory only — messages are lost if the process exits |
| No async | All operations are synchronous. `TransportClient::send()` blocks until a response is produced |
| Correlation | Response messages automatically set `correlation_id` to the request's `MessageId` |

### Determinism Constraints

| Constraint | Detail |
|---|---|
| PRNG quality | `deterministic_shuffle` uses a simple LCG — suitable for tie-breaking and shuffling, NOT for cryptography |
| Shuffle seed sensitivity | Different seeds produce different orderings. Same seed always produces the same ordering |
| `verify_deterministic_execution` | Runs the function twice with identical contexts and checks equality. Requires `T: PartialEq + Serialize` |
| `DeterminismGuard` | Hash-based — two structurally different values could theoretically hash-collide (SHA-256 collision, astronomically unlikely) |
| HashMap iteration | `deterministic_entries()` converts to `BTreeMap` for ordered iteration — clones all keys and values |

### Serialization Constraints

| Constraint | Detail |
|---|---|
| JSON only | All serialization uses `serde_json` — no binary formats |
| Canonical form | `to_canonical_json()` produces compact (non-pretty) JSON. Key ordering depends on the struct's field order and any `BTreeMap` usage |
| Round-trip safety | All types derive `Serialize + Deserialize` — round-trip through JSON is guaranteed |
| `DeterministicId` serialization | Serialized as a 64-character hex string in JSON for human readability and compatibility. Deserialized back to `[u8; 32]` |

### General Constraints

| Constraint | Detail |
|---|---|
| No `std::HashMap` iteration | Foundation uses `BTreeMap` for all ordered iteration. `HashMap` is used only where order doesn't matter (e.g., `AdjacencyList`'s internal node-to-index mapping) |
| No random number generation | There is no `rand` dependency. Pseudo-randomness comes only from deterministic seeds via `DeterministicContext` |
| No I/O | Foundation performs no file I/O, network I/O, or system calls. The only exception is `WallTimestamp::now()` for human-readable audit timestamps |
| No global state | No singletons, no thread-local storage, no lazy statics. All state is explicitly passed |
| Rust 1.70+ | Minimum supported Rust version. Uses edition 2021 |
| Overflow checks in release | `Cargo.toml` sets `overflow-checks = true` in the release profile — arithmetic overflow panics even in optimized builds |
| LTO in release | Thin LTO is enabled in the release profile for cross-crate inlining |

---

## Error Handling

All fallible operations return `FoundationResult<T>`, which is `Result<T, FoundationError>`. The error enum has nine variants:

| Variant | When It Occurs |
|---|---|
| `Overflow` | Arithmetic overflow, division by zero, timestamp overflow |
| `OutOfRange` | Non-finite float, index out of bounds, value outside valid range |
| `ValidationFailed` | Dimension mismatch, invalid state transition, missing required parameter, cycle in DAG |
| `SerializationError` | JSON serialization or deserialization failure |
| `HashError` | Hashing operation failure |
| `ContextError` | Execution context issues |
| `TransportError` | Message routing failure, no handler found |
| `IdError` | ID generation or parsing failure |
| `InternalError` | Catch-all for unexpected internal errors |

All errors implement `Display`, `Debug`, and `std::error::Error`. They carry a `String` message describing the specific failure.

---

## Dependencies

Foundation uses a minimal dependency set:

| Dependency | Purpose |
|---|---|
| `serde` + `serde_json` | Serialization and canonical JSON |
| `sha2` | SHA-256 hashing for deterministic IDs and content hashing |
| `chrono` | Wall-clock timestamps for audit events (human readability only) |
| `uuid` | UUID generation for message IDs |

No runtime, no async executor, no allocator customization, no platform-specific code.
