//! # osm-pathfinder binary
//!
//! A high-performance OpenStreetMap routing engine and navigation API.
//!
//! This executable parses OSM PBF files, constructs an in-memory road graph,
//! and serves shortest-path queries via a RESTful HTTP API.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use osm_pathfinder::{api, graph, osm, spatial, AppState};

/// Command-line arguments for osm-pathfinder.
#[derive(Parser, Debug)]
#[command(
    name = "osm-pathfinder",
    about = "High-performance OpenStreetMap routing engine",
    version
)]
struct Args {
    /// Path to the OSM PBF data file (e.g. data/cambodia-latest.osm.pbf).
    #[arg(short, long, env = "OSM_DATA_PATH")]
    data: Option<PathBuf>,

    /// Run with built-in Cambodia road network.
    #[arg(long, default_value_t = false)]
    demo: bool,

    /// Host address to bind the server to.
    #[arg(long, default_value = "127.0.0.1", env = "SERVER_HOST")]
    host: String,

    /// Port to bind the server to.
    #[arg(short, long, default_value_t = 3000, env = "SERVER_PORT")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!("osm-pathfinder v{}", env!("CARGO_PKG_VERSION"));

    let use_demo = args.demo || args.data.as_ref().map(|p| !p.exists()).unwrap_or(true);

    let road_graph = if use_demo {
        if let Some(ref path) = args.data {
            if !path.exists() {
                warn!(
                    path = %path.display(),
                    "OSM file not found. Falling back to built-in Cambodia road network"
                );
            }
        } else {
            info!("No OSM file specified (--data). Loading built-in Cambodia highway network");
        }
        graph::create_demo_graph()
    } else {
        let path = args.data.as_ref().unwrap();
        info!(path = %path.display(), "Loading OSM PBF data");
        osm::parse_pbf(path)
            .with_context(|| format!("Failed to parse OSM file: {}", path.display()))?
    };

    info!(
        nodes = road_graph.node_count(),
        edges = road_graph.edge_count(),
        "Road graph constructed"
    );

    // Build spatial index for coordinate snapping
    let spatial_index = spatial::SpatialIndex::new(&road_graph);
    info!("Spatial index built");

    // Create shared application state
    let state = Arc::new(AppState {
        road_graph,
        spatial_index,
    });

    // Build and start the HTTP server
    let app = api::create_router(state);

    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    info!(address = %addr, "Server listening");

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
