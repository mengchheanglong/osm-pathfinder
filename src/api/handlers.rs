//! HTTP request handlers for the routing API.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::pathfinding::{self, Algorithm, CostMetric, PathResult, RoutingOptions};
use crate::traffic;
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
    pub is_demo: bool,
    pub dataset_name: String,
    pub graph_version: String,
    pub cost_model_version: String,
}

/// GET /api/graph/stats
///
/// Returns metadata about the loaded road graph.
pub async fn graph_stats(State(state): State<Arc<AppState>>) -> Json<GraphStatsResponse> {
    Json(GraphStatsResponse {
        nodes: state.road_graph.node_count(),
        edges: state.road_graph.edge_count(),
        is_demo: state.is_demo,
        dataset_name: state.dataset_name.clone(),
        graph_version: state.graph_version.clone(),
        cost_model_version: state.cost_model_version.clone(),
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
    /// Cost metric optimization (distance vs. time).
    #[serde(default)]
    pub metric: CostMetric,
    /// Optional departure time in 24h format (e.g. "08:15", "17:30") for traffic simulation.
    pub departure_time: Option<String>,
    /// Vehicle profile: car, van, truck, motorcycle. Defaults to car.
    #[serde(default)]
    pub profile: Option<String>,
    /// Whether to include coordinates of visited nodes for wavefront visualization.
    #[serde(default)]
    pub include_explored: bool,
}

fn default_algorithm() -> Algorithm {
    Algorithm::Astar
}

/// Response body for the route calculation endpoint.
#[derive(Debug, Serialize)]
pub struct RouteResponse {
    /// GeoJSON-compatible path coordinates `[[lon, lat], ...]`.
    pub path: Vec<[f64; 2]>,
    /// Visited node coordinates during search expansion (GeoJSON format `[[lon, lat], ...]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub explored: Vec<[f64; 2]>,
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
    /// Metric optimized for ("distance" or "time").
    pub metric: String,
    /// Departure time if simulated.
    pub departure_time: Option<String>,
    /// Start node snapped coordinate.
    pub start_snapped: [f64; 2],
    /// End node snapped coordinate.
    pub end_snapped: [f64; 2],
    /// Whether this route was calculated on the demo synthetic graph.
    pub is_demo: bool,
    /// Loaded road graph version.
    pub graph_version: String,
    /// Cost model version.
    pub cost_model_version: String,
}

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /api/route
///
/// Calculates the shortest or fastest path between two geographic coordinates.
///
/// Supports time-dependent traffic modeling, multiple algorithms (Dijkstra, A*,
/// Bidirectional A*, Bidirectional Dijkstra), and search wavefront visualization.
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

    // Validate that Contraction Hierarchies is not combined with dynamic departure_time
    if request.algorithm == Algorithm::ContractionHierarchies && request.departure_time.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Contraction Hierarchies does not support dynamic departure_time (TDSP) because shortcut edge weights are precomputed statically. Use Dijkstra, A*, or Bidirectional search for time-dependent traffic routing, or remove departure_time.".to_string(),
            }),
        ));
    }

    let departure_minutes = request
        .departure_time
        .as_deref()
        .and_then(traffic::parse_time_of_day);

    let profile = request
        .profile
        .as_deref()
        .and_then(pathfinding::VehicleProfile::from_str_opt)
        .unwrap_or_default();

    let routing_options = RoutingOptions {
        metric: request.metric,
        profile,
        departure_minutes,
        collect_explored: request.include_explored,
    };

    info!(
        algorithm = %request.algorithm,
        metric = ?request.metric,
        departure_time = ?request.departure_time,
        start_node,
        end_node,
        "Running pathfinding query"
    );

    let result: Option<PathResult> = match request.algorithm {
        Algorithm::Dijkstra => pathfinding::dijkstra_search_with_options(
            &state.road_graph,
            start_node,
            end_node,
            &routing_options,
        ),
        Algorithm::Astar => pathfinding::astar_search_with_options(
            &state.road_graph,
            start_node,
            end_node,
            &routing_options,
        ),
        Algorithm::BidirectionalDijkstra => {
            pathfinding::bidirectional_dijkstra_search_with_options(
                &state.road_graph,
                start_node,
                end_node,
                &routing_options,
            )
        }
        Algorithm::BidirectionalAstar => pathfinding::bidirectional_astar_search_with_options(
            &state.road_graph,
            start_node,
            end_node,
            &routing_options,
        ),
        Algorithm::ContractionHierarchies => pathfinding::ch_search(
            &state.ch_graph,
            &state.road_graph,
            start_node,
            end_node,
            &routing_options,
        ),
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

            let explored: Vec<[f64; 2]> = path_result
                .explored_coordinates
                .iter()
                .map(|c| [c.lon, c.lat])
                .collect();

            let metric_str = match request.metric {
                CostMetric::Distance => "distance".to_string(),
                CostMetric::Time => "time".to_string(),
            };

            Ok(Json(RouteResponse {
                path,
                explored,
                distance_m: path_result.distance_m,
                duration_s: path_result.duration_s,
                nodes_visited: path_result.nodes_visited,
                query_time_ms: path_result.query_time_ms(),
                algorithm: path_result.algorithm.to_string(),
                metric: metric_str,
                departure_time: request.departure_time,
                start_snapped: [start_lon, start_lat],
                end_snapped: [end_lon, end_lat],
                is_demo: state.is_demo,
                graph_version: state.graph_version.clone(),
                cost_model_version: state.cost_model_version.clone(),
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

// ---------------------------------------------------------------------------
// Isochrone calculation
// ---------------------------------------------------------------------------

/// Request body for the isochrone calculation endpoint.
#[derive(Debug, Deserialize)]
pub struct IsochroneRequest {
    /// Starting latitude.
    pub lat: f64,
    /// Starting longitude.
    pub lon: f64,
    /// Travel time thresholds in minutes (defaults to `[10, 20, 30]`).
    #[serde(default = "default_isochrone_buckets")]
    pub buckets: Vec<u32>,
    /// Optional departure time in 24h format (e.g. "08:15").
    pub departure_time: Option<String>,
}

fn default_isochrone_buckets() -> Vec<u32> {
    vec![10, 20, 30]
}

/// Query parameters for GET /api/isochrone.
#[derive(Debug, Deserialize)]
pub struct IsochroneQuery {
    pub lat: f64,
    pub lon: f64,
    pub buckets: Option<String>,
    pub departure_time: Option<String>,
}

/// POST /api/isochrone
///
/// Computes reachability polygons (GeoJSON FeatureCollection) within travel time limits.
pub async fn calculate_isochrone(
    State(state): State<Arc<AppState>>,
    Json(request): Json<IsochroneRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    compute_isochrone_internal(
        &state,
        request.lat,
        request.lon,
        request.buckets,
        request.departure_time,
    )
    .await
}

/// GET /api/isochrone
///
/// Computes reachability polygons via URL query parameters.
pub async fn calculate_isochrone_get(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<IsochroneQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let buckets: Vec<u32> = match query.buckets {
        Some(s) => s
            .split(',')
            .filter_map(|part| part.trim().parse::<u32>().ok())
            .collect(),
        None => vec![10, 20, 30],
    };

    compute_isochrone_internal(&state, query.lat, query.lon, buckets, query.departure_time).await
}

async fn compute_isochrone_internal(
    state: &AppState,
    lat: f64,
    lon: f64,
    buckets: Vec<u32>,
    departure_time: Option<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let (start_node, start_lat, start_lon) = match state.spatial_index.nearest_node(lat, lon) {
        Some(result) => result,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Could not snap coordinate to road network".to_string(),
                }),
            ));
        }
    };

    let departure_minutes = departure_time
        .as_deref()
        .and_then(traffic::parse_time_of_day);

    let options = RoutingOptions {
        metric: CostMetric::Time,
        profile: pathfinding::VehicleProfile::Car,
        departure_minutes,
        collect_explored: false,
    };

    let valid_buckets = if buckets.is_empty() {
        vec![10, 20, 30]
    } else {
        buckets
    };

    info!(
        lat,
        lon,
        snapped_node = start_node,
        buckets = ?valid_buckets,
        "Computing isochrones"
    );

    match pathfinding::compute_isochrones(&state.road_graph, start_node, &valid_buckets, &options) {
        Some(result) => {
            let mut geojson = result.to_geojson();
            if let Some(obj) = geojson.as_object_mut() {
                if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
                    props.insert(
                        "snapped_coord".to_string(),
                        serde_json::json!([start_lon, start_lat]),
                    );
                }
            }
            Ok(Json(geojson))
        }
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to compute isochrone contours".to_string(),
            }),
        )),
    }
}

// ---------------------------------------------------------------------------
// Directed Cost Matrix
// ---------------------------------------------------------------------------

/// Geographic coordinate input for matrix requests.
#[derive(Debug, Deserialize)]
pub struct CoordinateInput {
    pub lat: f64,
    pub lon: f64,
}

/// Request body for the directed cost matrix endpoint.
#[derive(Debug, Deserialize)]
pub struct MatrixRequest {
    /// List of origin coordinates.
    pub origins: Vec<CoordinateInput>,
    /// List of destination coordinates.
    pub destinations: Vec<CoordinateInput>,
    /// Vehicle profile: car, van, truck, motorcycle. Defaults to car.
    #[serde(default)]
    pub profile: Option<String>,
    /// Optimization metric (distance vs time). Defaults to time.
    #[serde(default)]
    pub metric: CostMetric,
    /// Optional departure time in 24h format (e.g. "08:15", "17:30").
    pub departure_time: Option<String>,
}

/// POST /api/matrix
///
/// Computes an N x M directed travel duration and distance matrix.
///
/// Supports asymmetric network costs, vehicle profile road filtering and speed caps,
/// time-dependent traffic delays, and explicit null cells for unreachable components.
pub async fn calculate_matrix(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MatrixRequest>,
) -> Result<Json<pathfinding::MatrixResponse>, (StatusCode, Json<ErrorResponse>)> {
    if request.origins.is_empty() || request.destinations.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Origins and destinations must not be empty".to_string(),
            }),
        ));
    }

    let profile = request
        .profile
        .as_deref()
        .and_then(pathfinding::VehicleProfile::from_str_opt)
        .unwrap_or_default();

    let origins: Vec<crate::graph::Coordinate> = request
        .origins
        .iter()
        .map(|c| crate::graph::Coordinate::new(c.lat, c.lon))
        .collect();

    let destinations: Vec<crate::graph::Coordinate> = request
        .destinations
        .iter()
        .map(|c| crate::graph::Coordinate::new(c.lat, c.lon))
        .collect();

    info!(
        origins_count = origins.len(),
        destinations_count = destinations.len(),
        profile = profile.as_str(),
        metric = ?request.metric,
        "Computing directed cost matrix"
    );

    match pathfinding::compute_cost_matrix(
        &state.road_graph,
        &state.spatial_index,
        &origins,
        &destinations,
        profile,
        request.metric,
        request.departure_time.as_deref(),
    ) {
        Ok(matrix) => Ok(Json(matrix)),
        Err(pathfinding::MatrixError::OutOfBounds) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Coordinate outside routable network bounds".to_string(),
            }),
        )),
        Err(pathfinding::MatrixError::Unsnappable) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No road node found near coordinate within routable distance".to_string(),
            }),
        )),
    }
}
