//! Common types for pathfinding results and algorithm selection.

use serde::Serialize;
use std::time::Duration;

use crate::graph::Coordinate;

/// Available pathfinding algorithms.
#[derive(Debug, Clone, Copy, serde::Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    /// Dijkstra's algorithm — uniform-cost search (baseline).
    Dijkstra,
    /// A* search with Haversine great-circle heuristic.
    Astar,
    /// Bidirectional Dijkstra search.
    BidirectionalDijkstra,
    /// Bidirectional A* search with balanced heuristics.
    BidirectionalAstar,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Dijkstra => write!(f, "dijkstra"),
            Algorithm::Astar => write!(f, "astar"),
            Algorithm::BidirectionalDijkstra => write!(f, "bidirectional_dijkstra"),
            Algorithm::BidirectionalAstar => write!(f, "bidirectional_astar"),
        }
    }
}

/// Optimization criterion for pathfinding.
#[derive(Debug, Clone, Copy, serde::Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostMetric {
    /// Optimize for shortest physical distance (meters).
    #[default]
    Distance,
    /// Optimize for fastest travel time (seconds, incorporating time-dependent traffic delays).
    Time,
}

/// Optional configuration options for routing requests.
#[derive(Debug, Clone, Default)]
pub struct RoutingOptions {
    /// Optimization criterion: Distance vs. Time.
    pub metric: CostMetric,
    /// Optional departure time in minutes from midnight (0..1439).
    pub departure_minutes: Option<u32>,
    /// Whether to collect explored node coordinates for wavefront visualization.
    pub collect_explored: bool,
}

/// Result of a pathfinding query, including the path and benchmark metrics.
#[derive(Debug, Serialize)]
pub struct PathResult {
    /// Ordered list of internal node IDs forming the shortest path.
    pub path: Vec<u32>,
    /// Geographic coordinates along the path (for GeoJSON rendering).
    pub coordinates: Vec<Coordinate>,
    /// Explored coordinates during search expansion (for wavefront visualization).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explored_coordinates: Vec<Coordinate>,
    /// Total path distance in meters.
    pub distance_m: f64,
    /// Estimated travel duration in seconds.
    pub duration_s: f64,
    /// Number of nodes visited (expanded) during the search.
    pub nodes_visited: usize,
    /// Wall-clock time taken for the search.
    #[serde(serialize_with = "serialize_duration")]
    pub query_time: Duration,
    /// Algorithm used.
    pub algorithm: Algorithm,
}

fn serialize_duration<S: serde::Serializer>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_f64(duration.as_secs_f64() * 1000.0)
}

impl PathResult {
    /// Returns the query time in milliseconds.
    pub fn query_time_ms(&self) -> f64 {
        self.query_time.as_secs_f64() * 1000.0
    }
}
