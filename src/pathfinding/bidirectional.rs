//! Bidirectional shortest path algorithms.
//!
//! Implements bidirectional search (both Dijkstra and A*) which runs
//! simultaneous forward search from the source and backward search from
//! the destination, meeting in the middle.
//!
//! ## Time Complexity & Optimization
//!
//! By searching in two smaller radii $r$ rather than one large radius $2r$,
//! the search space volume is approximately halved ($\pi r^2 + \pi r^2 = 2\pi r^2$
//! vs. $\pi (2r)^2 = 4\pi r^2$), reducing memory and expanded nodes by 50%–75%.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;

use crate::geo::haversine;
use crate::graph::RoadGraph;

use super::types::{Algorithm, PathResult};

#[derive(Debug, Clone, Copy)]
struct BiQueueEntry {
    node: u32,
    priority: f64,
    cost: f64,
}

impl PartialEq for BiQueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for BiQueueEntry {}

impl PartialOrd for BiQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BiQueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(Ordering::Equal)
    }
}

/// Runs Bidirectional Dijkstra search.
pub fn bidirectional_dijkstra_search(
    graph: &RoadGraph,
    start: u32,
    end: u32,
) -> Option<PathResult> {
    run_bidirectional(graph, start, end, false)
}

/// Runs Bidirectional A* search with balanced Haversine heuristic.
pub fn bidirectional_astar_search(
    graph: &RoadGraph,
    start: u32,
    end: u32,
) -> Option<PathResult> {
    run_bidirectional(graph, start, end, true)
}

fn run_bidirectional(
    graph: &RoadGraph,
    start: u32,
    end: u32,
    use_heuristic: bool,
) -> Option<PathResult> {
    let start_time = Instant::now();
    let n = graph.node_count();

    if start as usize >= n || end as usize >= n {
        return None;
    }

    if start == end {
        let coord = graph.get_coord(start).copied()?;
        return Some(PathResult {
            path: vec![start],
            coordinates: vec![coord],
            distance_m: 0.0,
            duration_s: 0.0,
            nodes_visited: 1,
            query_time: start_time.elapsed(),
            algorithm: if use_heuristic {
                Algorithm::BidirectionalAstar
            } else {
                Algorithm::BidirectionalDijkstra
            },
        });
    }

    let start_coord = graph.get_coord(start)?;
    let end_coord = graph.get_coord(end)?;

    let mut dist_f: Vec<f64> = vec![f64::INFINITY; n];
    let mut dist_b: Vec<f64> = vec![f64::INFINITY; n];
    let mut dur_f: Vec<f64> = vec![f64::INFINITY; n];
    let mut dur_b: Vec<f64> = vec![f64::INFINITY; n];

    let mut parent_f: Vec<Option<u32>> = vec![None; n];
    let mut parent_b: Vec<Option<u32>> = vec![None; n];

    let mut visited_f: Vec<bool> = vec![false; n];
    let mut visited_b: Vec<bool> = vec![false; n];

    let mut heap_f = BinaryHeap::new();
    let mut heap_b = BinaryHeap::new();

    dist_f[start as usize] = 0.0;
    dur_f[start as usize] = 0.0;
    dist_b[end as usize] = 0.0;
    dur_b[end as usize] = 0.0;

    let h_f_init = if use_heuristic {
        haversine::distance(start_coord.lat, start_coord.lon, end_coord.lat, end_coord.lon)
    } else {
        0.0
    };
    let h_b_init = h_f_init;

    heap_f.push(BiQueueEntry {
        node: start,
        priority: h_f_init,
        cost: 0.0,
    });
    heap_b.push(BiQueueEntry {
        node: end,
        priority: h_b_init,
        cost: 0.0,
    });

    let mut best_cost = f64::INFINITY;
    let mut best_meeting_node: Option<u32> = None;
    let mut nodes_visited = 0;

    while !heap_f.is_empty() || !heap_b.is_empty() {
        // Step forward search
        if let Some(entry_f) = heap_f.pop() {
            let u = entry_f.node;
            if !visited_f[u as usize] {
                visited_f[u as usize] = true;
                nodes_visited += 1;

                // Check meeting condition
                if visited_b[u as usize] {
                    let total = dist_f[u as usize] + dist_b[u as usize];
                    if total < best_cost {
                        best_cost = total;
                        best_meeting_node = Some(u);
                    }
                }

                // If best cost is established and top keys exceed it, can terminate
                if entry_f.cost >= best_cost {
                    break;
                }

                for edge in graph.neighbors(u) {
                    let v = edge.target;
                    let new_dist = dist_f[u as usize] + edge.distance_m;

                    if new_dist < dist_f[v as usize] {
                        dist_f[v as usize] = new_dist;
                        dur_f[v as usize] = dur_f[u as usize] + edge.duration_s;
                        parent_f[v as usize] = Some(u);

                        if visited_b[v as usize] {
                            let potential_cost = new_dist + dist_b[v as usize];
                            if potential_cost < best_cost {
                                best_cost = potential_cost;
                                best_meeting_node = Some(v);
                            }
                        }

                        let h = if use_heuristic {
                            if let Some(c) = graph.get_coord(v) {
                                haversine::distance(c.lat, c.lon, end_coord.lat, end_coord.lon)
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };

                        heap_f.push(BiQueueEntry {
                            node: v,
                            priority: new_dist + h,
                            cost: new_dist,
                        });
                    }
                }
            }
        }

        // Step backward search
        if let Some(entry_b) = heap_b.pop() {
            let u = entry_b.node;
            if !visited_b[u as usize] {
                visited_b[u as usize] = true;
                nodes_visited += 1;

                if visited_f[u as usize] {
                    let total = dist_f[u as usize] + dist_b[u as usize];
                    if total < best_cost {
                        best_cost = total;
                        best_meeting_node = Some(u);
                    }
                }

                if entry_b.cost >= best_cost {
                    break;
                }

                // Reverse neighbors: incoming edges towards u (from w to u)
                for edge in graph.reverse_neighbors(u) {
                    let w = edge.target;
                    let new_dist = dist_b[u as usize] + edge.distance_m;

                    if new_dist < dist_b[w as usize] {
                        dist_b[w as usize] = new_dist;
                        dur_b[w as usize] = dur_b[u as usize] + edge.duration_s;
                        parent_b[w as usize] = Some(u);

                        if visited_f[w as usize] {
                            let potential_cost = dist_f[w as usize] + new_dist;
                            if potential_cost < best_cost {
                                best_cost = potential_cost;
                                best_meeting_node = Some(w);
                            }
                        }

                        let h = if use_heuristic {
                            if let Some(c) = graph.get_coord(w) {
                                haversine::distance(c.lat, c.lon, start_coord.lat, start_coord.lon)
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };

                        heap_b.push(BiQueueEntry {
                            node: w,
                            priority: new_dist + h,
                            cost: new_dist,
                        });
                    }
                }
            }
        }

        // Termination check: if both forward and backward queues top costs exceed best_cost
        let top_f = heap_f.peek().map(|e| e.cost).unwrap_or(f64::INFINITY);
        let top_b = heap_b.peek().map(|e| e.cost).unwrap_or(f64::INFINITY);
        if top_f + top_b >= best_cost && best_meeting_node.is_some() {
            break;
        }
    }

    let meeting_node = best_meeting_node?;
    if best_cost.is_infinite() {
        return None;
    }

    // Reconstruct path:
    // 1. Forward half: trace parent_f from meeting_node back to start
    let mut forward_path = Vec::new();
    let mut curr = meeting_node;
    while curr != start {
        forward_path.push(curr);
        curr = parent_f[curr as usize]?;
    }
    forward_path.push(start);
    forward_path.reverse();

    // 2. Backward half: trace parent_b from meeting_node to end
    let mut backward_path = Vec::new();
    let mut curr_b = meeting_node;
    while curr_b != end {
        curr_b = parent_b[curr_b as usize]?;
        backward_path.push(curr_b);
    }

    // Combine path
    let mut full_path = forward_path;
    full_path.extend(backward_path);

    let coordinates: Vec<_> = full_path
        .iter()
        .filter_map(|&id| graph.get_coord(id).copied())
        .collect();

    let total_duration = dur_f[meeting_node as usize] + dur_b[meeting_node as usize];

    Some(PathResult {
        path: full_path,
        coordinates,
        distance_m: best_cost,
        duration_s: total_duration,
        nodes_visited,
        query_time: start_time.elapsed(),
        algorithm: if use_heuristic {
            Algorithm::BidirectionalAstar
        } else {
            Algorithm::BidirectionalDijkstra
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    fn build_diamond_graph() -> RoadGraph {
        let mut builder = GraphBuilder::new();
        builder.add_node(100, 11.550, 104.920); // 0
        builder.add_node(101, 11.555, 104.925); // 1
        builder.add_node(102, 11.545, 104.925); // 2
        builder.add_node(103, 11.550, 104.930); // 3

        builder.add_way(&[100, 101, 103], false, 60.0);
        builder.add_way(&[100, 102, 103], false, 60.0);
        builder.build()
    }

    #[test]
    fn test_bidirectional_dijkstra() {
        let graph = build_diamond_graph();
        let res = bidirectional_dijkstra_search(&graph, 0, 3).unwrap();
        assert_eq!(*res.path.first().unwrap(), 0);
        assert_eq!(*res.path.last().unwrap(), 3);
        assert!(res.distance_m > 0.0);
    }

    #[test]
    fn test_bidirectional_astar() {
        let graph = build_diamond_graph();
        let res = bidirectional_astar_search(&graph, 0, 3).unwrap();
        assert_eq!(*res.path.first().unwrap(), 0);
        assert_eq!(*res.path.last().unwrap(), 3);
        assert!(res.distance_m > 0.0);
    }

    #[test]
    fn test_bidirectional_matches_unidirectional() {
        let graph = build_diamond_graph();
        let uni = crate::pathfinding::dijkstra_search(&graph, 0, 3).unwrap();
        let bi = bidirectional_dijkstra_search(&graph, 0, 3).unwrap();
        let bi_a = bidirectional_astar_search(&graph, 0, 3).unwrap();

        assert!((uni.distance_m - bi.distance_m).abs() < 1e-2);
        assert!((uni.distance_m - bi_a.distance_m).abs() < 1e-2);
    }
}
