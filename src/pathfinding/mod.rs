//! Shortest path algorithms for the road graph.
//!
//! Provides implementations of Dijkstra's algorithm and A* search,
//! with a common interface for comparing their performance.

mod astar;
mod bidirectional;
mod dijkstra;
mod types;

pub use astar::astar_search;
pub use bidirectional::{bidirectional_astar_search, bidirectional_dijkstra_search};
pub use dijkstra::dijkstra_search;
pub use types::{Algorithm, PathResult};
