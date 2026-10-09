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
    pub ch_graph: std::sync::Arc<pathfinding::ChGraph>,
    pub is_demo: bool,
    pub dataset_name: String,
    pub graph_version: String,
    pub cost_model_version: String,
}

impl AppState {
    pub fn new_demo(
        road_graph: graph::RoadGraph,
        spatial_index: spatial::SpatialIndex,
        ch_graph: std::sync::Arc<pathfinding::ChGraph>,
    ) -> Self {
        Self {
            road_graph,
            spatial_index,
            ch_graph,
            is_demo: true,
            dataset_name: "demo-cambodia".to_string(),
            graph_version: "demo-cambodia-v1.0".to_string(),
            cost_model_version: "tdsp-profiles-v1.0".to_string(),
        }
    }
}

