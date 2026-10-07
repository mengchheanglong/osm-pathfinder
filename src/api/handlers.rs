//! HTTP request handlers for the routing API.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::pathfinding::{self, Algorithm, PathResult};
use crate::AppState;

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

/// Response for the health check endpoint.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// GET /api/health
///
/// Returns the service status and version.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ---------------------------------------------------------------------------
// Graph statistics
// ---------------------------------------------------------------------------

/// Response for the graph statistics endpoint.
#[derive(Serialize)]
pub struct GraphStatsResponse {
    pub nodes: usize,
    pub edges: usize,
}

/// GET /api/graph/stats
///
/// Returns metadata about the loaded road graph.
pub async fn graph_stats(State(state): State<Arc<AppState>>) -> Json<GraphStatsResponse> {
    Json(GraphStatsResponse {
        nodes: state.road_graph.node_count(),
        edges: state.road_graph.edge_count(),
    })
}

// ---------------------------------------------------------------------------
// Route calculation
// ---------------------------------------------------------------------------

/// Request body for the route calculation endpoint.
#[derive(Deserialize)]
pub struct RouteRequest {
    /// Starting point latitude.
    pub start_lat: f64,
    /// Starting point longitude.
    pub start_lon: f64,
    /// Destination latitude.
    pub end_lat: f64,
    /// Destination longitude.
    pub end_lon: f64,
    /// Algorithm to use. Defaults to A* if not specified.
    #[serde(default = "default_algorithm")]
    pub algorithm: Algorithm,
}

fn default_algorithm() -> Algorithm {
    Algorithm::Astar
}

/// Response body for the route calculation endpoint.
#[derive(Serialize)]
pub struct RouteResponse {
    /// GeoJSON-compatible path coordinates `[[lon, lat], ...]`.
    pub path: Vec<[f64; 2]>,
    /// Total distance in meters.
    pub distance_m: f64,
    /// Estimated travel duration in seconds.
    pub duration_s: f64,
    /// Number of nodes visited during the search.
    pub nodes_visited: usize,
    /// Query execution time in milliseconds.
    pub query_time_ms: f64,
    /// Algorithm used.
    pub algorithm: String,
    /// Start node snapped coordinate.
    pub start_snapped: [f64; 2],
    /// End node snapped coordinate.
    pub end_snapped: [f64; 2],
}

/// Error response body.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /api/route
///
/// Calculates the shortest path between two geographic coordinates.
///
/// The provided coordinates are snapped to the nearest graph nodes
/// via the spatial index before running the selected pathfinding algorithm.
pub async fn calculate_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RouteRequest>,
) -> Result<Json<RouteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Snap start coordinate to nearest graph node
    let (start_node, start_lat, start_lon) = state
        .spatial_index
        .nearest_node(request.start_lat, request.start_lon)
        .ok_or_else(|| {
            warn!(
                lat = request.start_lat,
                lon = request.start_lon,
                "No graph node found near start coordinate"
            );
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "No road node found near the start coordinate".to_string(),
                }),
            )
        })?;

    // Snap end coordinate to nearest graph node
    let (end_node, end_lat, end_lon) = state
        .spatial_index
        .nearest_node(request.end_lat, request.end_lon)
        .ok_or_else(|| {
            warn!(
                lat = request.end_lat,
                lon = request.end_lon,
                "No graph node found near end coordinate"
            );
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "No road node found near the end coordinate".to_string(),
                }),
            )
        })?;

    info!(
        algorithm = %request.algorithm,
        start_node,
        end_node,
        "Running pathfinding query"
    );

    // Run the selected algorithm
    let result: Option<PathResult> = match request.algorithm {
        Algorithm::Dijkstra => pathfinding::dijkstra_search(&state.road_graph, start_node, end_node),
        Algorithm::Astar => pathfinding::astar_search(&state.road_graph, start_node, end_node),
    };

    match result {
        Some(path_result) => {
            info!(
                algorithm = %path_result.algorithm,
                distance_m = path_result.distance_m,
                duration_s = path_result.duration_s,
                nodes_visited = path_result.nodes_visited,
                query_time_ms = path_result.query_time_ms(),
                "Route found"
            );

            // Convert coordinates to GeoJSON format [lon, lat]
            let path: Vec<[f64; 2]> = path_result
                .coordinates
                .iter()
                .map(|c| [c.lon, c.lat])
                .collect();

            Ok(Json(RouteResponse {
                path,
                distance_m: path_result.distance_m,
                duration_s: path_result.duration_s,
                nodes_visited: path_result.nodes_visited,
                query_time_ms: path_result.query_time_ms(),
                algorithm: path_result.algorithm.to_string(),
                start_snapped: [start_lon, start_lat],
                end_snapped: [end_lon, end_lat],
            }))
        }
        None => {
            warn!(start_node, end_node, "No path found between nodes");
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "No route found between the specified coordinates".to_string(),
                }),
            ))
        }
    }
}
