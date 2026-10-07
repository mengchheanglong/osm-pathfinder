//! OpenStreetMap data parsing.
//!
//! Handles reading and interpreting OSM Protocol Buffer Format (PBF) files,
//! extracting road network data and constructing the in-memory graph.

mod parser;

pub use parser::parse_pbf;
