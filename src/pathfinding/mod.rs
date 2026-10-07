//! Shortest path algorithms for the road graph.
//!
//! Provides implementations of Dijkstra's algorithm and A* search,
//! with a common interface for comparing their performance.

mod astar;
mod bidirectional;
mod dijkstra;
mod types;

pub use astar::{astar_search, astar_search_with_options};
pub use bidirectional::{
    bidirectional_astar_search, bidirectional_astar_search_with_options,
    bidirectional_dijkstra_search, bidirectional_dijkstra_search_with_options,
};
pub use dijkstra::{dijkstra_search, dijkstra_search_with_options};
pub use types::{Algorithm, CostMetric, PathResult, RoutingOptions};
