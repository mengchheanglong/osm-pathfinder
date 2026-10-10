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
use tracing::info;

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
    #[arg(short, long, default_value_t = 8000, env = "SERVER_PORT")]
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

    let road_graph = if let Some(ref path) = args.data {
        if !path.exists() {
            anyhow::bail!("OSM PBF data file not found: {}", path.display());
        }
        info!(path = %path.display(), "Loading OSM PBF data");
        osm::parse_pbf(path)
            .with_context(|| format!("Failed to parse OSM file: {}", path.display()))?
    } else {
        info!("Loading built-in Cambodia highway network");
        graph::create_demo_graph()
    };

    info!(
        nodes = road_graph.node_count(),
        edges = road_graph.edge_count(),
        "Road graph constructed"
    );

    // Build spatial index for coordinate snapping
    let spatial_index = spatial::SpatialIndex::new(&road_graph);
    info!("Spatial index built");

    // Preprocess Contraction Hierarchies (shortcuts for sub-millisecond queries)
    info!("Preprocessing Contraction Hierarchies...");
    let ch_start = std::time::Instant::now();
    let ch_graph = Arc::new(osm_pathfinder::pathfinding::build_contraction_hierarchies(
        &road_graph,
    ));
    info!(
        shortcuts = ch_graph.shortcut_count(),
        elapsed_ms = ch_start.elapsed().as_millis(),
        "Contraction Hierarchies preprocessed"
    );

    let (is_demo, dataset_name, graph_version) = if let Some(ref path) = args.data {
        (
            false,
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format!("pbf-{}", path.display()),
        )
    } else {
        (
            true,
            "demo-cambodia".to_string(),
            "demo-cambodia-v1.0".to_string(),
        )
    };

    // Create shared application state
    let state = Arc::new(AppState {
        road_graph,
        spatial_index,
        ch_graph,
        is_demo,
        dataset_name,
        graph_version,
        cost_model_version: "tdsp-profiles-v1.0".to_string(),
    });

    // Build and start the HTTP server
    let app = api::create_router(state);

    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    info!(address = %addr, "Server listening");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
