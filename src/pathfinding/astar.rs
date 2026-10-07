//! A* shortest path algorithm with Haversine heuristic.
//!
//! Implements A* search using the Haversine great-circle distance as
//! the admissible heuristic function h(n).
//!
//! ## Time Complexity
//!
//! O((V + E) log V) worst case, but typically much faster than Dijkstra
//! due to heuristic pruning of the search space.
//!
//! ## Admissibility Proof
//!
//! The Haversine heuristic is admissible because the straight-line
//! (great-circle) distance between two points on Earth is always less
//! than or equal to the actual road distance:
//!
//! ```text
//! h(n) = haversine(n, goal) ≤ road_distance(n, goal) = d*(n)
//! ```
//!
//! This holds due to the triangle inequality on a sphere: no path along
//! road segments can be shorter than the direct arc between two points.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;

use crate::geo::haversine;
use crate::graph::RoadGraph;

use super::types::{Algorithm, CostMetric, PathResult, RoutingOptions};
use crate::graph::Coordinate;
use crate::traffic;

/// A node in the A* priority queue with f(n) = g(n) + h(n).
#[derive(Debug, Clone, Copy)]
struct AStarEntry {
    node: u32,
    /// f(n) = g(n) + h(n)
    f_cost: f64,
    /// g(n) = actual cost from start to this node
    g_cost: f64,
}

impl PartialEq for AStarEntry {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for AStarEntry {}

impl PartialOrd for AStarEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Computes the admissible heuristic $h(n)$ based on the routing metric.
fn compute_heuristic(coord: &Coordinate, goal_lat: f64, goal_lon: f64, metric: CostMetric) -> f64 {
    let dist_m = haversine::distance(coord.lat, coord.lon, goal_lat, goal_lon);
    match metric {
        CostMetric::Distance => dist_m,
        CostMetric::Time => dist_m / traffic::MAX_NETWORK_SPEED_MS,
    }
}

/// Runs A* search on the road graph with Haversine heuristic.
///
/// Finds the shortest path from `start` to `end` (internal node IDs).
/// The heuristic function h(n) uses the Haversine formula to calculate
/// the straight-line distance to the goal, providing an admissible
/// lower bound that dramatically prunes the search space.
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
pub fn astar_search(graph: &RoadGraph, start: u32, end: u32) -> Option<PathResult> {
    astar_search_with_options(graph, start, end, &RoutingOptions::default())
}

/// Runs A* search on the road graph with customizable routing options.
pub fn astar_search_with_options(
    graph: &RoadGraph,
    start: u32,
    end: u32,
    options: &RoutingOptions,
) -> Option<PathResult> {
    let start_time = Instant::now();
    let num_nodes = graph.node_count();

    if start as usize >= num_nodes || end as usize >= num_nodes {
        return None;
    }

    let goal_coord = graph.get_coord(end)?;
    let goal_lat = goal_coord.lat;
    let goal_lon = goal_coord.lon;

    let mut g_cost_arr: Vec<f64> = vec![f64::INFINITY; num_nodes];
    let mut dist_arr: Vec<f64> = vec![f64::INFINITY; num_nodes];
    let mut dur_arr: Vec<f64> = vec![f64::INFINITY; num_nodes];
    let mut parent: Vec<Option<u32>> = vec![None; num_nodes];
    let mut visited: Vec<bool> = vec![false; num_nodes];
    let mut nodes_visited: usize = 0;
    let mut explored_coordinates = Vec::new();

    g_cost_arr[start as usize] = 0.0;
    dist_arr[start as usize] = 0.0;
    dur_arr[start as usize] = 0.0;

    let start_coord = graph.get_coord(start)?;
    let h_start = compute_heuristic(start_coord, goal_lat, goal_lon, options.metric);

    let mut heap = BinaryHeap::new();
    heap.push(AStarEntry {
        node: start,
        f_cost: h_start,
        g_cost: 0.0,
    });

    while let Some(AStarEntry {
        node,
        f_cost: _,
        g_cost,
    }) = heap.pop()
    {
        if visited[node as usize] {
            continue;
        }
        visited[node as usize] = true;
        nodes_visited += 1;

        if options.collect_explored && explored_coordinates.len() < 1200 {
            if let Some(c) = graph.get_coord(node) {
                explored_coordinates.push(*c);
            }
        }

        if node == end {
            break;
        }

        if g_cost > g_cost_arr[node as usize] {
            continue;
        }

        for edge in graph.neighbors(node) {
            let target_coord = match graph.get_coord(edge.target) {
                Some(c) => c,
                None => continue,
            };

            let edge_duration = {
                let multiplier =
                    traffic::congestion_multiplier(target_coord, options.departure_minutes);
                edge.duration_s * multiplier
            };

            let edge_cost = match options.metric {
                CostMetric::Distance => edge.distance_m,
                CostMetric::Time => edge_duration,
            };

            let new_g = g_cost_arr[node as usize] + edge_cost;

            if new_g < g_cost_arr[edge.target as usize] {
                g_cost_arr[edge.target as usize] = new_g;
                dist_arr[edge.target as usize] = dist_arr[node as usize] + edge.distance_m;
                dur_arr[edge.target as usize] = dur_arr[node as usize] + edge_duration;
                parent[edge.target as usize] = Some(node);

                let h = compute_heuristic(target_coord, goal_lat, goal_lon, options.metric);

                heap.push(AStarEntry {
                    node: edge.target,
                    f_cost: new_g + h,
                    g_cost: new_g,
                });
            }
        }
    }

    let query_time = start_time.elapsed();

    if dist_arr[end as usize].is_infinite() {
        return None;
    }

    let (path, coordinates) = reconstruct_path(graph, &parent, start, end);

    Some(PathResult {
        path,
        coordinates,
        explored_coordinates,
        distance_m: dist_arr[end as usize],
        duration_s: dur_arr[end as usize],
        nodes_visited,
        query_time,
        algorithm: Algorithm::Astar,
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
            None => break,
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
        // Diamond graph (same as Dijkstra tests for comparison):
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

        builder.add_way(&[100, 101, 103], false, 60.0);
        builder.add_way(&[100, 102, 103], false, 60.0);

        builder.build()
    }

    #[test]
    fn test_astar_finds_path() {
        let graph = build_test_graph();
        let result = astar_search(&graph, 0, 3);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.distance_m > 0.0);
        assert!(result.nodes_visited > 0);
        assert_eq!(*result.path.first().unwrap(), 0);
        assert_eq!(*result.path.last().unwrap(), 3);
    }

    #[test]
    fn test_astar_same_node() {
        let graph = build_test_graph();
        let result = astar_search(&graph, 0, 0);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.distance_m, 0.0);
        assert_eq!(result.path, vec![0]);
    }

    #[test]
    fn test_astar_optimal_same_as_dijkstra() {
        let graph = build_test_graph();

        let dijkstra_result = crate::pathfinding::dijkstra::dijkstra_search(&graph, 0, 3).unwrap();
        let astar_result = astar_search(&graph, 0, 3).unwrap();

        // A* should find the same optimal distance
        let diff = (dijkstra_result.distance_m - astar_result.distance_m).abs();
        assert!(
            diff < 1.0, // within 1 meter tolerance
            "A* distance ({:.1}m) should match Dijkstra ({:.1}m)",
            astar_result.distance_m,
            dijkstra_result.distance_m,
        );
    }

    #[test]
    fn test_astar_algorithm_label() {
        let graph = build_test_graph();
        let result = astar_search(&graph, 0, 3).unwrap();
        assert_eq!(result.algorithm, Algorithm::Astar);
    }

    #[test]
    fn test_astar_no_path() {
        let mut builder = GraphBuilder::new();
        builder.add_node(1, 11.55, 104.92);
        builder.add_node(2, 13.36, 103.84);
        let graph = builder.build();

        let result = astar_search(&graph, 0, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_astar_visits_fewer_nodes() {
        // Create a larger grid-like graph where A* should prune significantly
        let mut builder = GraphBuilder::new();

        // Create a 5x5 grid of nodes
        // Goal is at the far corner, so A* should focus search direction
        for i in 0..5 {
            for j in 0..5 {
                let osm_id = (i * 5 + j + 1) as i64;
                let lat = 11.0 + (i as f64) * 0.01;
                let lon = 104.0 + (j as f64) * 0.01;
                builder.add_node(osm_id, lat, lon);
            }
        }

        // Connect grid horizontally and vertically
        for i in 0..5 {
            for j in 0..5 {
                let id = (i * 5 + j + 1) as i64;
                if j < 4 {
                    builder.add_way(&[id, id + 1], false, 50.0);
                }
                if i < 4 {
                    builder.add_way(&[id, id + 5], false, 50.0);
                }
            }
        }

        let graph = builder.build();

        let dijkstra = crate::pathfinding::dijkstra::dijkstra_search(&graph, 0, 24).unwrap();
        let astar = astar_search(&graph, 0, 24).unwrap();

        // A* should visit fewer or equal nodes than Dijkstra
        assert!(
            astar.nodes_visited <= dijkstra.nodes_visited,
            "A* ({} nodes) should visit ≤ Dijkstra ({} nodes)",
            astar.nodes_visited,
            dijkstra.nodes_visited,
        );
    }
}
