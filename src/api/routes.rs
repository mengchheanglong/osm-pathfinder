//! Axum router configuration.
//!
//! Defines all HTTP routes and attaches shared application state.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::AppState;

use super::handlers;

/// Creates the Axum router with all API routes and middleware.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(handlers::health_check))
        .route("/api/route", post(handlers::calculate_route))
        .route(
            "/api/isochrone",
            post(handlers::calculate_isochrone).get(handlers::calculate_isochrone_get),
        )
        .route("/api/graph/stats", get(handlers::graph_stats))
        .fallback_service(ServeDir::new("frontend"))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
