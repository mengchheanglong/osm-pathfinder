//! Core graph types for the road network representation.

use serde::Serialize;

/// A geographic coordinate (latitude, longitude) in WGS84.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

impl Coordinate {
    /// Creates a new coordinate from latitude and longitude.
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

/// A directed edge in the road graph.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    /// Target node (internal compact ID).
    pub target: u32,
    /// Distance in meters.
    pub distance_m: f64,
    /// Estimated travel duration in seconds (based on road type speed).
    pub duration_s: f64,
}

/// The in-memory road graph built from OpenStreetMap data.
///
/// Uses a flat adjacency list representation for cache-friendly traversal.
/// Node IDs are compact unsigned integers (0..N), mapped from OSM node IDs
/// via `osm_id_map`.
pub struct RoadGraph {
    /// Adjacency list: `adjacency[node_id]` contains all outgoing edges.
    pub adjacency: Vec<Vec<Edge>>,
    /// Reverse adjacency list: `reverse_adjacency[node_id]` contains all incoming edges.
    pub reverse_adjacency: Vec<Vec<Edge>>,
    /// Coordinates for each node, indexed by internal node ID.
    pub coords: Vec<Coordinate>,
    /// Mapping from OSM node IDs (i64) to internal compact IDs (u32).
    osm_id_map: std::collections::HashMap<i64, u32>,
}

impl RoadGraph {
    /// Creates a new road graph with pre-allocated capacity.
    pub fn new(
        adjacency: Vec<Vec<Edge>>,
        reverse_adjacency: Vec<Vec<Edge>>,
        coords: Vec<Coordinate>,
        osm_id_map: std::collections::HashMap<i64, u32>,
    ) -> Self {
        Self {
            adjacency,
            reverse_adjacency,
            coords,
            osm_id_map,
        }
    }

    /// Returns the total number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns the total number of directed edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.adjacency.iter().map(|edges| edges.len()).sum()
    }

    /// Returns the coordinate of a node by its internal ID.
    pub fn get_coord(&self, node_id: u32) -> Option<&Coordinate> {
        self.coords.get(node_id as usize)
    }

    /// Returns the outgoing edges of a node by its internal ID.
    pub fn neighbors(&self, node_id: u32) -> &[Edge] {
        self.adjacency
            .get(node_id as usize)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the incoming edges of a node by its internal ID (for backward search).
    pub fn reverse_neighbors(&self, node_id: u32) -> &[Edge] {
        self.reverse_adjacency
            .get(node_id as usize)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Looks up the internal node ID for a given OSM node ID.
    pub fn get_internal_id(&self, osm_id: i64) -> Option<u32> {
        self.osm_id_map.get(&osm_id).copied()
    }
}
