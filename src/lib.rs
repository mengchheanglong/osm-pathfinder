//! # osm-pathfinder
//!
//! A high-performance OpenStreetMap routing engine and navigation API.
//!
//! This library provides graph representations of road networks,
//! spatial indexing, OSM PBF parsing, and pathfinding algorithms (Dijkstra, A*).

pub mod api;
pub mod geo;
pub mod graph;
pub mod osm;
pub mod pathfinding;
pub mod spatial;
pub mod traffic;

/// Shared application state passed to all API handlers.
pub struct AppState {
    pub road_graph: graph::RoadGraph,
    pub spatial_index: spatial::SpatialIndex,
}
