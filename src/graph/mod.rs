//! Graph data structures for representing road networks.
//!
//! This module provides the core types for storing road network topology
//! as an adjacency list with parallel coordinate storage.

mod builder;
pub mod demo;
mod types;

pub use builder::GraphBuilder;
pub use demo::create_demo_graph;
pub use types::{Coordinate, Edge, RoadGraph};
