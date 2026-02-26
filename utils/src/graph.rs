//! M17: Graph Helpers – deterministic graph algorithms.

use crate::graph_types::AdjacencyList;
use foundation::errors::{FoundationError, FoundationResult};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Breadth-first search from a source node. Returns nodes in BFS order.
pub fn bfs(graph: &AdjacencyList, source: &str) -> FoundationResult<Vec<String>> {
    if graph.neighbors(source).is_none() {
        return Err(FoundationError::ValidationFailed(format!(
            "source node '{}' not in graph",
            source
        )));
    }

    let node_count = graph.adjacency.len();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::with_capacity(node_count);
    let mut result = Vec::with_capacity(node_count);

    visited.insert(source.to_string());
    queue.push_back(source.to_string());

    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = graph.neighbors(&node) {
            let mut sorted_neighbors: Vec<_> = neighbors.iter().collect();
            sorted_neighbors.sort_by_key(|(name, _)| name.clone());
            for (neighbor, _) in sorted_neighbors {
                if visited.insert(neighbor.clone()) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
        result.push(node);
    }

    Ok(result)
}

/// Depth-first search from a source node. Returns nodes in DFS order.
pub fn dfs(graph: &AdjacencyList, source: &str) -> FoundationResult<Vec<String>> {
    if graph.neighbors(source).is_none() {
        return Err(FoundationError::ValidationFailed(format!(
            "source node '{}' not in graph",
            source
        )));
    }

    let mut visited = BTreeSet::new();
    let mut result = Vec::new();
    dfs_recursive(graph, source, &mut visited, &mut result);
    Ok(result)
}

fn dfs_recursive(
    graph: &AdjacencyList,
    node: &str,
    visited: &mut BTreeSet<String>,
    result: &mut Vec<String>,
) {
    visited.insert(node.to_string());
    result.push(node.to_string());
    if let Some(neighbors) = graph.neighbors(node) {
        let mut sorted_neighbors: Vec<_> = neighbors.iter().collect();
        sorted_neighbors.sort_by_key(|(name, _)| name.clone());
        for (neighbor, _) in sorted_neighbors {
            if !visited.contains(neighbor) {
                dfs_recursive(graph, neighbor, visited, result);
            }
        }
    }
}

/// Topological sort (Kahn's algorithm). Returns an error if the graph has a cycle.
pub fn topological_sort(graph: &AdjacencyList) -> FoundationResult<Vec<String>> {
    // Compute in-degrees
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    for node in graph.adjacency.keys() {
        in_degree.entry(node.clone()).or_insert(0);
    }
    for neighbors in graph.adjacency.values() {
        for (neighbor, _) in neighbors {
            *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
        }
    }

    // Start with zero in-degree nodes (BTreeSet for deterministic ordering)
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut zero_in: BTreeSet<String> = BTreeSet::new();
    for (node, &deg) in &in_degree {
        if deg == 0 {
            zero_in.insert(node.clone());
        }
    }
    for node in &zero_in {
        queue.push_back(node.clone());
    }

    let mut result = Vec::with_capacity(in_degree.len());
    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = graph.adjacency.get(&node) {
            let mut sorted: Vec<_> = neighbors.iter().collect();
            sorted.sort_by_key(|(name, _)| name.clone());
            for (neighbor, _) in sorted {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
        result.push(node);
    }

    if result.len() != in_degree.len() {
        return Err(FoundationError::ValidationFailed(
            "graph contains a cycle; topological sort not possible".into(),
        ));
    }

    Ok(result)
}

/// Dijkstra's shortest path algorithm using a BinaryHeap for O((V+E) log V).
/// Returns (distances, predecessors) from the source node.
pub fn dijkstra(
    graph: &AdjacencyList,
    source: &str,
) -> FoundationResult<(BTreeMap<String, f64>, BTreeMap<String, String>)> {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    if graph.neighbors(source).is_none() {
        return Err(FoundationError::ValidationFailed(format!(
            "source node '{}' not in graph",
            source
        )));
    }

    /// Priority queue entry: wraps f64 distance for min-heap ordering.
    /// Ties are broken by node name for determinism.
    #[derive(PartialEq)]
    struct State {
        dist: f64,
        node: String,
    }
    impl Eq for State {}
    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            // Reverse for min-heap, then break ties deterministically by name
            other
                .dist
                .partial_cmp(&self.dist)
                .unwrap_or(Ordering::Equal)
                .then_with(|| other.node.cmp(&self.node))
        }
    }

    let mut dist: BTreeMap<String, f64> = BTreeMap::new();
    let mut pred: BTreeMap<String, String> = BTreeMap::new();
    let mut heap = BinaryHeap::new();

    for node in graph.adjacency.keys() {
        dist.insert(node.clone(), f64::MAX);
    }
    dist.insert(source.to_string(), 0.0);
    heap.push(State {
        dist: 0.0,
        node: source.to_string(),
    });

    while let Some(State {
        dist: current_dist,
        node: current,
    }) = heap.pop()
    {
        // Skip if we already found a shorter path
        if let Some(&best) = dist.get(&current) {
            if current_dist > best {
                continue;
            }
        }

        if let Some(neighbors) = graph.neighbors(&current) {
            for (neighbor, weight) in neighbors {
                let alt = current_dist + weight.value();
                let current_best = dist.get(neighbor).copied().unwrap_or(f64::MAX);
                if alt < current_best {
                    dist.insert(neighbor.clone(), alt);
                    pred.insert(neighbor.clone(), current.clone());
                    heap.push(State {
                        dist: alt,
                        node: neighbor.clone(),
                    });
                }
            }
        }
    }

    Ok((dist, pred))
}

/// Reconstruct the shortest path from Dijkstra's predecessor map.
pub fn reconstruct_path(
    predecessors: &BTreeMap<String, String>,
    source: &str,
    target: &str,
) -> Option<Vec<String>> {
    if source == target {
        return Some(vec![source.to_string()]);
    }
    let mut path = Vec::new();
    let mut current = target.to_string();
    while current != source {
        path.push(current.clone());
        match predecessors.get(&current) {
            Some(prev) => current = prev.clone(),
            None => return None,
        }
    }
    path.push(source.to_string());
    path.reverse();
    Some(path)
}

/// Find connected components in an undirected graph.
pub fn connected_components(graph: &AdjacencyList) -> Vec<Vec<String>> {
    let mut visited = BTreeSet::new();
    let mut components = Vec::new();

    for node in graph.adjacency.keys() {
        if !visited.contains(node) {
            let mut component = Vec::new();
            let mut stack = vec![node.clone()];
            while let Some(n) = stack.pop() {
                if visited.insert(n.clone()) {
                    component.push(n.clone());
                    if let Some(neighbors) = graph.neighbors(&n) {
                        for (neighbor, _) in neighbors {
                            if !visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }
                }
            }
            component.sort(); // deterministic ordering
            components.push(component);
        }
    }

    components.sort(); // deterministic ordering of components
    components
}

/// Detect if a directed graph has a cycle (using DFS coloring).
pub fn has_cycle(graph: &AdjacencyList) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut color: BTreeMap<String, Color> = BTreeMap::new();
    for node in graph.adjacency.keys() {
        color.insert(node.clone(), Color::White);
    }

    fn visit(
        node: &str,
        graph: &AdjacencyList,
        color: &mut BTreeMap<String, Color>,
    ) -> bool {
        color.insert(node.to_string(), Color::Gray);
        if let Some(neighbors) = graph.neighbors(node) {
            let mut sorted: Vec<_> = neighbors.iter().collect();
            sorted.sort_by_key(|(name, _)| name.clone());
            for (neighbor, _) in sorted {
                match color.get(neighbor) {
                    Some(Color::Gray) => return true,
                    Some(Color::White) => {
                        if visit(neighbor, graph, color) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        color.insert(node.to_string(), Color::Black);
        false
    }

    let nodes: Vec<String> = graph.adjacency.keys().cloned().collect();
    for node in &nodes {
        if color.get(node) == Some(&Color::White)
            && visit(node, graph, &mut color)
        {
            return true;
        }
    }
    false
}

/// Compute the degree of each node (out-degree for directed graphs).
pub fn node_degrees(graph: &AdjacencyList) -> BTreeMap<String, usize> {
    let mut degrees = BTreeMap::new();
    for (node, neighbors) in &graph.adjacency {
        degrees.insert(node.clone(), neighbors.len());
    }
    degrees
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::primitives::SafeFloat;

    fn sample_graph() -> AdjacencyList {
        let mut g = AdjacencyList::new();
        g.add_node("a");
        g.add_node("b");
        g.add_node("c");
        g.add_node("d");
        g.add_edge("a", "b", SafeFloat::ONE);
        g.add_edge("a", "c", SafeFloat::new(2.0).unwrap());
        g.add_edge("b", "d", SafeFloat::ONE);
        g.add_edge("c", "d", SafeFloat::ONE);
        g
    }

    #[test]
    fn bfs_basic() {
        let g = sample_graph();
        let result = bfs(&g, "a").unwrap();
        assert_eq!(result[0], "a");
        assert!(result.contains(&"b".to_string()));
        assert!(result.contains(&"c".to_string()));
        assert!(result.contains(&"d".to_string()));
    }

    #[test]
    fn dfs_basic() {
        let g = sample_graph();
        let result = dfs(&g, "a").unwrap();
        assert_eq!(result[0], "a");
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn topological_sort_basic() {
        let g = sample_graph();
        let result = topological_sort(&g).unwrap();
        // 'a' must come before 'b' and 'c'
        let pos_a = result.iter().position(|x| x == "a").unwrap();
        let pos_b = result.iter().position(|x| x == "b").unwrap();
        let pos_d = result.iter().position(|x| x == "d").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_d);
    }

    #[test]
    fn topological_sort_cycle() {
        let mut g = AdjacencyList::new();
        g.add_node("a");
        g.add_node("b");
        g.add_edge("a", "b", SafeFloat::ONE);
        g.add_edge("b", "a", SafeFloat::ONE);
        assert!(topological_sort(&g).is_err());
    }

    #[test]
    fn dijkstra_basic() {
        let g = sample_graph();
        let (dist, _pred) = dijkstra(&g, "a").unwrap();
        assert_eq!(*dist.get("a").unwrap(), 0.0);
        assert_eq!(*dist.get("b").unwrap(), 1.0);
        assert_eq!(*dist.get("d").unwrap(), 2.0);
    }

    #[test]
    fn reconstruct_path_basic() {
        let g = sample_graph();
        let (_, pred) = dijkstra(&g, "a").unwrap();
        let path = reconstruct_path(&pred, "a", "d").unwrap();
        assert_eq!(path.first().unwrap(), "a");
        assert_eq!(path.last().unwrap(), "d");
    }

    #[test]
    fn has_cycle_false() {
        let g = sample_graph();
        assert!(!has_cycle(&g));
    }

    #[test]
    fn has_cycle_true() {
        let mut g = AdjacencyList::new();
        g.add_node("a");
        g.add_node("b");
        g.add_edge("a", "b", SafeFloat::ONE);
        g.add_edge("b", "a", SafeFloat::ONE);
        assert!(has_cycle(&g));
    }

    #[test]
    fn connected_components_basic() {
        let mut g = AdjacencyList::new();
        g.add_node("a");
        g.add_node("b");
        g.add_node("c");
        g.add_edge("a", "b", SafeFloat::ONE);
        // c is isolated
        let components = connected_components(&g);
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn node_degrees_basic() {
        let g = sample_graph();
        let degrees = node_degrees(&g);
        assert_eq!(*degrees.get("a").unwrap(), 2); // a -> b, a -> c
    }

    #[test]
    fn bfs_invalid_source() {
        let g = sample_graph();
        assert!(bfs(&g, "nonexistent").is_err());
    }
}
