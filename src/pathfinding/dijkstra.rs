//! Dijkstra's shortest path algorithm.
//!
//! Implements the classic priority-queue-based Dijkstra's algorithm for
//! finding the shortest path between two nodes in a weighted graph.
//!
//! ## Time Complexity
//!
//! O((V + E) log V) using a binary heap priority queue.
//!
//! ## Characteristics
//!
//! - Explores nodes uniformly in all directions (no heuristic guidance).
//! - Guarantees optimal shortest path.
//! - Serves as the baseline for comparison against A*.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;

use crate::graph::RoadGraph;

use super::types::{Algorithm, PathResult};

/// A node in the priority queue with its accumulated cost.
///
/// Implements `Ord` to create a min-heap (BinaryHeap is a max-heap by default,
/// so we reverse the comparison).
#[derive(Debug, Clone, Copy)]
struct QueueEntry {
    node: u32,
    cost: f64,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for QueueEntry {}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Runs Dijkstra's algorithm on the road graph.
///
/// Finds the shortest path from `start` to `end` (internal node IDs).
///
/// # Arguments
///
/// * `graph` - The road graph to search.
/// * `start` - Starting node (internal ID).
/// * `end` - Destination node (internal ID).
///
/// # Returns
///
/// `Some(PathResult)` containing the shortest path and metrics,
/// or `None` if no path exists between the two nodes.
pub fn dijkstra_search(graph: &RoadGraph, start: u32, end: u32) -> Option<PathResult> {
    let start_time = Instant::now();
    let num_nodes = graph.node_count();

    // Distance to each node (infinity initially)
    let mut dist: Vec<f64> = vec![f64::INFINITY; num_nodes];
    // Duration to each node
    let mut dur: Vec<f64> = vec![f64::INFINITY; num_nodes];
    // Parent pointer for path reconstruction
    let mut parent: Vec<Option<u32>> = vec![None; num_nodes];
    // Track visited nodes
    let mut visited: Vec<bool> = vec![false; num_nodes];
    let mut nodes_visited: usize = 0;

    dist[start as usize] = 0.0;
    dur[start as usize] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(QueueEntry {
        node: start,
        cost: 0.0,
    });

    while let Some(QueueEntry { node, cost }) = heap.pop() {
        // Skip if already visited
        if visited[node as usize] {
            continue;
        }
        visited[node as usize] = true;
        nodes_visited += 1;

        // Early termination: found the destination
        if node == end {
            break;
        }

        // Skip stale entries
        if cost > dist[node as usize] {
            continue;
        }

        // Explore neighbors
        for edge in graph.neighbors(node) {
            let new_dist = dist[node as usize] + edge.distance_m;

            if new_dist < dist[edge.target as usize] {
                dist[edge.target as usize] = new_dist;
                dur[edge.target as usize] = dur[node as usize] + edge.duration_s;
                parent[edge.target as usize] = Some(node);

                heap.push(QueueEntry {
                    node: edge.target,
                    cost: new_dist,
                });
            }
        }
    }

    let query_time = start_time.elapsed();

    // Check if destination was reached
    if dist[end as usize].is_infinite() {
        return None;
    }

    // Reconstruct the path
    let (path, coordinates) = reconstruct_path(graph, &parent, start, end);

    Some(PathResult {
        path,
        coordinates,
        distance_m: dist[end as usize],
        duration_s: dur[end as usize],
        nodes_visited,
        query_time,
        algorithm: Algorithm::Dijkstra,
    })
}

/// Reconstructs the shortest path by backtracking through parent pointers.
fn reconstruct_path(
    graph: &RoadGraph,
    parent: &[Option<u32>],
    start: u32,
    end: u32,
) -> (Vec<u32>, Vec<crate::graph::Coordinate>) {
    let mut path = Vec::new();
    let mut current = end;

    loop {
        path.push(current);
        if current == start {
            break;
        }
        match parent[current as usize] {
            Some(p) => current = p,
            None => break, // Should not happen if path exists
        }
    }

    path.reverse();

    let coordinates: Vec<_> = path
        .iter()
        .filter_map(|&node_id| graph.get_coord(node_id).copied())
        .collect();

    (path, coordinates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    fn build_test_graph() -> RoadGraph {
        // Simple diamond graph:
        //     1
        //    / \
        //   0   3
        //    \ /
        //     2
        let mut builder = GraphBuilder::new();
        builder.add_node(100, 11.550, 104.920); // Node 0
        builder.add_node(101, 11.555, 104.925); // Node 1
        builder.add_node(102, 11.545, 104.925); // Node 2
        builder.add_node(103, 11.550, 104.930); // Node 3

        // Upper path: 0 -> 1 -> 3
        builder.add_way(&[100, 101, 103], false, 60.0);
        // Lower path: 0 -> 2 -> 3
        builder.add_way(&[100, 102, 103], false, 60.0);

        builder.build()
    }

    #[test]
    fn test_dijkstra_finds_path() {
        let graph = build_test_graph();
        let result = dijkstra_search(&graph, 0, 3);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.distance_m > 0.0);
        assert!(result.nodes_visited > 0);
        assert_eq!(*result.path.first().unwrap(), 0);
        assert_eq!(*result.path.last().unwrap(), 3);
    }

    #[test]
    fn test_dijkstra_same_node() {
        let graph = build_test_graph();
        let result = dijkstra_search(&graph, 0, 0);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.distance_m, 0.0);
        assert_eq!(result.path, vec![0]);
    }

    #[test]
    fn test_dijkstra_no_path() {
        // Create disconnected graph
        let mut builder = GraphBuilder::new();
        builder.add_node(1, 11.55, 104.92);
        builder.add_node(2, 13.36, 103.84);
        // No edges between them
        let graph = builder.build();

        let result = dijkstra_search(&graph, 0, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_dijkstra_algorithm_label() {
        let graph = build_test_graph();
        let result = dijkstra_search(&graph, 0, 3).unwrap();
        assert_eq!(result.algorithm, Algorithm::Dijkstra);
    }
}
