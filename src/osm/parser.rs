//! PBF file parser for OpenStreetMap data.
//!
//! Implements a two-pass parsing strategy:
//! 1. **Pass 1**: Scan all ways tagged as highways, collect referenced node IDs.
//! 2. **Pass 2**: Read node coordinates for referenced nodes and build edges.
//!
//! This approach avoids loading all OSM data into memory — only road-related
//! nodes and ways are retained.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use osmpbf::{ElementReader, Element};
use tracing::{debug, info};

use crate::graph::{GraphBuilder, RoadGraph};

/// Highway types to include in the road network, with estimated speed limits (km/h).
const HIGHWAY_SPEEDS: &[(&str, f64)] = &[
    ("motorway", 120.0),
    ("motorway_link", 80.0),
    ("trunk", 100.0),
    ("trunk_link", 60.0),
    ("primary", 80.0),
    ("primary_link", 50.0),
    ("secondary", 60.0),
    ("secondary_link", 40.0),
    ("tertiary", 50.0),
    ("tertiary_link", 30.0),
    ("unclassified", 40.0),
    ("residential", 30.0),
    ("service", 20.0),
    ("living_street", 15.0),
];

/// Parses an OSM PBF file and constructs a [`RoadGraph`].
///
/// # Arguments
///
/// * `path` - Path to the `.osm.pbf` file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn parse_pbf(path: &Path) -> Result<RoadGraph> {
    info!(path = %path.display(), "Starting OSM PBF parsing");

    // Build a lookup map for highway speeds
    let speed_map: HashMap<&str, f64> = HIGHWAY_SPEEDS.iter().copied().collect();

    // --- Pass 1: Collect all ways and their node references ---
    info!("Pass 1: Scanning highway ways");

    let reader = ElementReader::from_path(path)
        .with_context(|| format!("Failed to open PBF file: {}", path.display()))?;

    // Stores: (node_ids, is_oneway, speed_kmh)
    let mut ways: Vec<(Vec<i64>, bool, f64)> = Vec::new();
    let mut referenced_nodes: HashSet<i64> = HashSet::new();

    reader.for_each(|element| {
        if let Element::Way(way) = element {
            // Check for highway tag
            let mut highway_type: Option<&str> = None;
            let mut is_oneway = false;

            for (key, value) in way.tags() {
                match key {
                    "highway" => highway_type = Some(value),
                    "oneway" => {
                        is_oneway = matches!(value, "yes" | "true" | "1");
                    }
                    _ => {}
                }
            }

            if let Some(hw_type) = highway_type {
                if let Some(&speed) = speed_map.get(hw_type) {
                    let node_ids: Vec<i64> = way.refs().collect();
                    for &id in &node_ids {
                        referenced_nodes.insert(id);
                    }

                    // Motorways are typically one-way
                    if hw_type == "motorway" || hw_type == "motorway_link" {
                        is_oneway = true;
                    }

                    ways.push((node_ids, is_oneway, speed));
                }
            }
        }
    }).context("Failed to read PBF elements (pass 1)")?;

    info!(
        ways = ways.len(),
        referenced_nodes = referenced_nodes.len(),
        "Pass 1 complete"
    );

    // --- Pass 2: Read node coordinates and build the graph ---
    info!("Pass 2: Reading node coordinates and building graph");

    let mut builder = GraphBuilder::with_capacity(referenced_nodes.len());

    let reader = ElementReader::from_path(path)
        .with_context(|| format!("Failed to reopen PBF file: {}", path.display()))?;

    reader.for_each(|element| {
        if let Element::Node(node) = element {
            if referenced_nodes.contains(&node.id()) {
                builder.add_node(node.id(), node.lat(), node.lon());
            }
        }
        if let Element::DenseNode(node) = element {
            if referenced_nodes.contains(&node.id) {
                builder.add_node(node.id, node.lat(), node.lon());
            }
        }
    }).context("Failed to read PBF elements (pass 2)")?;

    debug!("Node coordinates loaded, adding edges");

    // Add all road ways as edges
    for (node_ids, is_oneway, speed) in &ways {
        builder.add_way(node_ids, *is_oneway, *speed);
    }

    let graph = builder.build();

    info!(
        nodes = graph.node_count(),
        edges = graph.edge_count(),
        "PBF parsing complete"
    );

    Ok(graph)
}
