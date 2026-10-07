//! Graph construction from parsed OSM data.
//!
//! Provides a builder pattern for incrementally constructing a [`RoadGraph`]
//! from OSM nodes and ways.

use std::collections::HashMap;

use super::types::{Coordinate, Edge, RoadGraph};
use crate::geo::haversine;

/// Incrementally builds a [`RoadGraph`] from OSM elements.
pub struct GraphBuilder {
    /// Maps OSM node IDs to internal compact IDs.
    osm_id_map: HashMap<i64, u32>,
    /// Node coordinates indexed by internal ID.
    coords: Vec<Coordinate>,
    /// Adjacency list being constructed.
    adjacency: Vec<Vec<Edge>>,
    /// Counter for generating internal node IDs.
    next_id: u32,
}

impl GraphBuilder {
    /// Creates a new empty graph builder.
    pub fn new() -> Self {
        Self {
            osm_id_map: HashMap::new(),
            coords: Vec::new(),
            adjacency: Vec::new(),
            next_id: 0,
        }
    }

    /// Creates a new graph builder with estimated capacity hints.
    pub fn with_capacity(estimated_nodes: usize) -> Self {
        Self {
            osm_id_map: HashMap::with_capacity(estimated_nodes),
            coords: Vec::with_capacity(estimated_nodes),
            adjacency: Vec::with_capacity(estimated_nodes),
            next_id: 0,
        }
    }

    /// Registers an OSM node and returns its internal compact ID.
    ///
    /// If the node was already registered, returns the existing ID.
    pub fn add_node(&mut self, osm_id: i64, lat: f64, lon: f64) -> u32 {
        if let Some(&id) = self.osm_id_map.get(&osm_id) {
            // Update coordinates if the node already exists
            self.coords[id as usize] = Coordinate::new(lat, lon);
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.osm_id_map.insert(osm_id, id);
        self.coords.push(Coordinate::new(lat, lon));
        self.adjacency.push(Vec::new());

        id
    }

    /// Checks whether an OSM node ID has been registered.
    pub fn has_node(&self, osm_id: i64) -> bool {
        self.osm_id_map.contains_key(&osm_id)
    }

    /// Returns the internal ID for an OSM node, if registered.
    pub fn get_internal_id(&self, osm_id: i64) -> Option<u32> {
        self.osm_id_map.get(&osm_id).copied()
    }

    /// Adds a road segment (way) as edges between consecutive nodes.
    ///
    /// `node_osm_ids` are the OSM node IDs forming the way.
    /// `is_oneway` controls whether reverse edges are created.
    /// `speed_kmh` is the assumed travel speed for duration calculation.
    pub fn add_way(
        &mut self,
        node_osm_ids: &[i64],
        is_oneway: bool,
        speed_kmh: f64,
    ) {
        let speed_ms = speed_kmh / 3.6; // Convert km/h to m/s

        for window in node_osm_ids.windows(2) {
            let from_osm = window[0];
            let to_osm = window[1];

            let from_id = match self.get_internal_id(from_osm) {
                Some(id) => id,
                None => continue,
            };
            let to_id = match self.get_internal_id(to_osm) {
                Some(id) => id,
                None => continue,
            };

            let from_coord = self.coords[from_id as usize];
            let to_coord = self.coords[to_id as usize];

            let distance_m = haversine::distance(
                from_coord.lat,
                from_coord.lon,
                to_coord.lat,
                to_coord.lon,
            );

            let duration_s = if speed_ms > 0.0 {
                distance_m / speed_ms
            } else {
                distance_m / (50.0 / 3.6) // fallback: 50 km/h
            };

            // Forward edge
            self.adjacency[from_id as usize].push(Edge {
                target: to_id,
                distance_m,
                duration_s,
            });

            // Reverse edge (if not one-way)
            if !is_oneway {
                self.adjacency[to_id as usize].push(Edge {
                    target: from_id,
                    distance_m,
                    duration_s,
                });
            }
        }
    }

    /// Consumes the builder and produces the final [`RoadGraph`].
    pub fn build(self) -> RoadGraph {
        RoadGraph::new(self.adjacency, self.coords, self.osm_id_map)
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut builder = GraphBuilder::new();
        let id1 = builder.add_node(100, 11.5, 104.9);
        let id2 = builder.add_node(200, 13.3, 103.8);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert!(builder.has_node(100));
        assert!(builder.has_node(200));
        assert!(!builder.has_node(300));
    }

    #[test]
    fn test_add_node_idempotent() {
        let mut builder = GraphBuilder::new();
        let id1 = builder.add_node(100, 11.5, 104.9);
        let id2 = builder.add_node(100, 11.5, 104.9);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_add_way_bidirectional() {
        let mut builder = GraphBuilder::new();
        builder.add_node(1, 11.556, 104.928);
        builder.add_node(2, 11.560, 104.930);
        builder.add_node(3, 11.565, 104.935);

        builder.add_way(&[1, 2, 3], false, 50.0);

        let graph = builder.build();
        // Node 0 (OSM 1) should have edge to node 1 (OSM 2)
        assert_eq!(graph.neighbors(0).len(), 1);
        assert_eq!(graph.neighbors(0)[0].target, 1);
        // Node 1 (OSM 2) should have edges to both 0 and 2 (bidirectional)
        assert_eq!(graph.neighbors(1).len(), 2);
    }

    #[test]
    fn test_add_way_oneway() {
        let mut builder = GraphBuilder::new();
        builder.add_node(1, 11.556, 104.928);
        builder.add_node(2, 11.560, 104.930);

        builder.add_way(&[1, 2], true, 50.0);

        let graph = builder.build();
        assert_eq!(graph.neighbors(0).len(), 1); // forward edge
        assert_eq!(graph.neighbors(1).len(), 0); // no reverse edge
    }
}
