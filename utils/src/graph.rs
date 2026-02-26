//! M17: Graph Helpers – deterministic graph algorithms.
//!
//! All algorithms operate on the compact index-based [`AdjacencyList`] representation.
//! Internally they use `Vec<bool>` for visited sets, `Vec<f64>` for distances, and
//! `usize` node indices throughout the hot loop, converting back to `String` node IDs
//! only at the return boundary.

use crate::graph_types::AdjacencyList;
use foundation::errors::{FoundationError, FoundationResult};
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

/// Breadth-first search from a source node. Returns nodes in BFS order.
pub fn bfs(graph: &AdjacencyList, source: &str) -> FoundationResult<Vec<String>> {
    let src = graph.node_index(source).ok_or_else(|| {
        FoundationError::ValidationFailed(format!("source node '{}' not in graph", source))
    })?;

    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut queue = VecDeque::with_capacity(n);
    let mut result = Vec::with_capacity(n);

    visited[src] = true;
    queue.push_back(src);

    while let Some(node) = queue.pop_front() {
        result.push(graph.node_name(node).to_string());
        // Sort neighbors by name for deterministic traversal order
        let mut neighbors: Vec<(usize, f64)> = graph.neighbors_idx(node).to_vec();
        neighbors.sort_unstable_by(|a, b| graph.node_name(a.0).cmp(graph.node_name(b.0)));
        for (neighbor, _) in neighbors {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }

    Ok(result)
}

/// Depth-first search from a source node. Returns nodes in DFS pre-order.
/// Uses an explicit stack to avoid stack overflow on deep graphs.
pub fn dfs(graph: &AdjacencyList, source: &str) -> FoundationResult<Vec<String>> {
    let src = graph.node_index(source).ok_or_else(|| {
        FoundationError::ValidationFailed(format!("source node '{}' not in graph", source))
    })?;

    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut result = Vec::with_capacity(n);
    let mut stack = vec![src];

    while let Some(node) = stack.pop() {
        if visited[node] {
            continue;
        }
        visited[node] = true;
        result.push(graph.node_name(node).to_string());
        // Push neighbors in reverse sorted-by-name order so the first
        // alphabetical neighbor is popped (and visited) next.
        let mut neighbors: Vec<usize> =
            graph.neighbors_idx(node).iter().map(|&(n, _)| n).collect();
        neighbors.sort_unstable_by(|&a, &b| graph.node_name(a).cmp(graph.node_name(b)));
        for &neighbor in neighbors.iter().rev() {
            if !visited[neighbor] {
                stack.push(neighbor);
            }
        }
    }

    Ok(result)
}

/// Topological sort (Kahn's algorithm). Returns an error if the graph has a cycle.
pub fn topological_sort(graph: &AdjacencyList) -> FoundationResult<Vec<String>> {
    let n = graph.node_count();
    let mut in_degree = vec![0usize; n];

    for src in 0..n {
        for &(dst, _) in graph.neighbors_idx(src) {
            in_degree[dst] += 1;
        }
    }

    // Seed with zero-in-degree nodes, sorted by name for determinism
    let mut zero_in: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    zero_in.sort_unstable_by(|&a, &b| graph.node_name(a).cmp(graph.node_name(b)));

    let mut queue: VecDeque<usize> = zero_in.into_iter().collect();
    let mut result = Vec::with_capacity(n);

    while let Some(node) = queue.pop_front() {
        let mut neighbors: Vec<(usize, f64)> = graph.neighbors_idx(node).to_vec();
        neighbors.sort_unstable_by(|a, b| graph.node_name(a.0).cmp(graph.node_name(b.0)));
        for (neighbor, _) in neighbors {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
        result.push(graph.node_name(node).to_string());
    }

    if result.len() != n {
        return Err(FoundationError::ValidationFailed(
            "graph contains a cycle; topological sort not possible".into(),
        ));
    }

    Ok(result)
}

/// Dijkstra's shortest path algorithm using a BinaryHeap for O((V+E) log V).
/// Returns (distances, predecessors) from the source node.
///
/// Internally uses `Vec<f64>` for distances and `Vec<Option<usize>>` for
/// predecessors, converting to `BTreeMap<String, _>` only at the return boundary.
pub fn dijkstra(
    graph: &AdjacencyList,
    source: &str,
) -> FoundationResult<(BTreeMap<String, f64>, BTreeMap<String, String>)> {
    use std::cmp::Ordering;

    let src = graph.node_index(source).ok_or_else(|| {
        FoundationError::ValidationFailed(format!("source node '{}' not in graph", source))
    })?;

    let n = graph.node_count();

    // Min-heap entry: lower distance has higher priority.
    // Ties broken by node index for determinism.
    #[derive(PartialEq)]
    struct State {
        dist: f64,
        node: usize,
    }
    impl Eq for State {}
    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .dist
                .partial_cmp(&self.dist)
                .unwrap_or(Ordering::Equal)
                .then_with(|| other.node.cmp(&self.node))
        }
    }

    let mut dist = vec![f64::MAX; n];
    let mut pred: Vec<Option<usize>> = vec![None; n];
    let mut heap = BinaryHeap::new();

    dist[src] = 0.0;
    heap.push(State {
        dist: 0.0,
        node: src,
    });

    while let Some(State {
        dist: current_dist,
        node: current,
    }) = heap.pop()
    {
        if current_dist > dist[current] {
            continue;
        }
        for &(neighbor, weight) in graph.neighbors_idx(current) {
            let alt = current_dist + weight;
            if alt < dist[neighbor] {
                dist[neighbor] = alt;
                pred[neighbor] = Some(current);
                heap.push(State {
                    dist: alt,
                    node: neighbor,
                });
            }
        }
    }

    // Convert to BTreeMap for the public API
    let mut dist_map = BTreeMap::new();
    let mut pred_map = BTreeMap::new();
    for i in 0..n {
        dist_map.insert(graph.node_name(i).to_string(), dist[i]);
        if let Some(p) = pred[i] {
            pred_map.insert(
                graph.node_name(i).to_string(),
                graph.node_name(p).to_string(),
            );
        }
    }

    Ok((dist_map, pred_map))
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
    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut components = Vec::new();

    for start in 0..n {
        if !visited[start] {
            let mut component = Vec::new();
            let mut stack = vec![start];
            while let Some(node) = stack.pop() {
                if !visited[node] {
                    visited[node] = true;
                    component.push(graph.node_name(node).to_string());
                    for &(neighbor, _) in graph.neighbors_idx(node) {
                        if !visited[neighbor] {
                            stack.push(neighbor);
                        }
                    }
                }
            }
            component.sort(); // deterministic ordering within component
            components.push(component);
        }
    }

    components.sort(); // deterministic ordering of components
    components
}

/// Detect if a directed graph has a cycle (using DFS coloring).
pub fn has_cycle(graph: &AdjacencyList) -> bool {
    let n = graph.node_count();

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut color = vec![Color::White; n];

    fn visit(node: usize, graph: &AdjacencyList, color: &mut [Color]) -> bool {
        color[node] = Color::Gray;
        for &(neighbor, _) in graph.neighbors_idx(node) {
            match color[neighbor] {
                Color::Gray => return true,
                Color::White => {
                    if visit(neighbor, graph, color) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        color[node] = Color::Black;
        false
    }

    for node in 0..n {
        if color[node] == Color::White && visit(node, graph, &mut color) {
            return true;
        }
    }
    false
}

/// Compute the degree of each node (out-degree for directed graphs).
pub fn node_degrees(graph: &AdjacencyList) -> BTreeMap<String, usize> {
    let mut degrees = BTreeMap::new();
    for i in 0..graph.node_count() {
        degrees.insert(
            graph.node_name(i).to_string(),
            graph.neighbors_idx(i).len(),
        );
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
