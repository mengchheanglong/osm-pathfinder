//! HTTP API layer for the routing engine.
//!
//! Provides the Axum router configuration and request handlers
//! for the REST API endpoints.

mod handlers;
mod routes;

pub use routes::create_router;
