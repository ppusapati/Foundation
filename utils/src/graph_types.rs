//! M4: Graph-Domain Compound Types – types specific to graph operations.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::id::{DeterministicId, EdgeDomain, GraphDomain, NodeDomain};
use foundation::primitives::SafeFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A traversal path through a graph: an ordered list of node IDs and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<DeterministicId<NodeDomain>>,
    pub edges: Vec<DeterministicId<EdgeDomain>>,
    pub total_weight: SafeFloat,
}

impl GraphPath {
    pub fn empty() -> Self {
        GraphPath {
            nodes: Vec::new(),
            edges: Vec::new(),
            total_weight: SafeFloat::ZERO,
        }
    }

    pub fn new(
        nodes: Vec<DeterministicId<NodeDomain>>,
        edges: Vec<DeterministicId<EdgeDomain>>,
        total_weight: SafeFloat,
    ) -> FoundationResult<Self> {
        if !nodes.is_empty() && !edges.is_empty() && edges.len() != nodes.len() - 1 {
            return Err(FoundationError::ValidationFailed(
                "edges count must be nodes count - 1".into(),
            ));
        }
        Ok(GraphPath {
            nodes,
            edges,
            total_weight,
        })
    }

    pub fn length(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// A subgraph view: a subset of nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphView {
    pub graph_id: DeterministicId<GraphDomain>,
    pub node_ids: Vec<DeterministicId<NodeDomain>>,
    pub edge_ids: Vec<DeterministicId<EdgeDomain>>,
    pub label: String,
}

impl SubgraphView {
    pub fn new(
        graph_id: DeterministicId<GraphDomain>,
        label: &str,
    ) -> Self {
        SubgraphView {
            graph_id,
            node_ids: Vec::new(),
            edge_ids: Vec::new(),
            label: label.to_string(),
        }
    }

    pub fn add_node(&mut self, id: DeterministicId<NodeDomain>) {
        if !self.node_ids.contains(&id) {
            self.node_ids.push(id);
        }
    }

    pub fn add_edge(&mut self, id: DeterministicId<EdgeDomain>) {
        if !self.edge_ids.contains(&id) {
            self.edge_ids.push(id);
        }
    }
}

/// A query filter for graph traversals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuery {
    pub label_filter: Option<String>,
    pub relation_filter: Option<String>,
    pub max_depth: Option<usize>,
    pub min_weight: Option<SafeFloat>,
    pub max_weight: Option<SafeFloat>,
    pub limit: Option<usize>,
}

impl GraphQuery {
    pub fn new() -> Self {
        GraphQuery {
            label_filter: None,
            relation_filter: None,
            max_depth: None,
            min_weight: None,
            max_weight: None,
            limit: None,
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label_filter = Some(label.to_string());
        self
    }

    pub fn with_relation(mut self, relation: &str) -> Self {
        self.relation_filter = Some(relation.to_string());
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for GraphQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// An adjacency list representation of a graph for algorithmic operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjacencyList {
    /// Map from node ID hex -> list of (neighbor ID hex, edge weight).
    pub adjacency: BTreeMap<String, Vec<(String, SafeFloat)>>,
}

impl AdjacencyList {
    pub fn new() -> Self {
        AdjacencyList {
            adjacency: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: &str) {
        self.adjacency.entry(node_id.to_string()).or_default();
    }

    pub fn add_edge(&mut self, from: &str, to: &str, weight: SafeFloat) {
        self.adjacency
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), weight));
    }

    pub fn neighbors(&self, node_id: &str) -> Option<&Vec<(String, SafeFloat)>> {
        self.adjacency.get(node_id)
    }

    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }
}

impl Default for AdjacencyList {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: SafeFloat,
    pub density: SafeFloat,
    pub is_connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::id::DeterministicId;

    #[test]
    fn graph_path_empty() {
        let path = GraphPath::empty();
        assert!(path.is_empty());
        assert_eq!(path.length(), 0);
    }

    #[test]
    fn graph_path_validation() {
        let n1: DeterministicId<NodeDomain> = DeterministicId::from_content("a");
        let n2: DeterministicId<NodeDomain> = DeterministicId::from_content("b");
        let e1: DeterministicId<EdgeDomain> = DeterministicId::from_content("e1");
        let path = GraphPath::new(vec![n1, n2], vec![e1], SafeFloat::ONE).unwrap();
        assert_eq!(path.length(), 1);
    }

    #[test]
    fn adjacency_list_basic() {
        let mut adj = AdjacencyList::new();
        adj.add_node("a");
        adj.add_node("b");
        adj.add_edge("a", "b", SafeFloat::ONE);
        assert_eq!(adj.node_count(), 2);
        assert_eq!(adj.edge_count(), 1);
        assert_eq!(adj.neighbors("a").unwrap().len(), 1);
    }

    #[test]
    fn graph_query_builder() {
        let q = GraphQuery::new()
            .with_label("person")
            .with_max_depth(3)
            .with_limit(10);
        assert_eq!(q.label_filter, Some("person".into()));
        assert_eq!(q.max_depth, Some(3));
        assert_eq!(q.limit, Some(10));
    }

    #[test]
    fn subgraph_view() {
        let gid: DeterministicId<GraphDomain> = DeterministicId::from_content("g1");
        let mut sg = SubgraphView::new(gid, "subset");
        sg.add_node(DeterministicId::from_content("n1"));
        sg.add_node(DeterministicId::from_content("n1")); // duplicate
        assert_eq!(sg.node_ids.len(), 1);
    }
}
