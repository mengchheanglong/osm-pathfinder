# AI Agent Instructions for osm-pathfinder

Welcome to the osm-pathfinder project. This document provides essential instructions, context, and conventions for AI coding agents contributing to this repository.

## Project Overview
- **osm-pathfinder**: A high-performance OpenStreetMap routing engine built in Rust
- **Web Framework**: Axum with Tokio async runtime
- **Pathfinding Algorithms**: Dijkstra, A* (Haversine heuristic)
- **Data Source**: OSM PBF parsing for real-world road network data

## Architecture
- Modular Rust workspace with clear separation of concerns
- `src/api/` - HTTP API layer (Axum handlers, routes)
- `src/graph/` - Graph data structures (adjacency list, node/edge types)
- `src/pathfinding/` - Algorithm implementations (Dijkstra, A*)
- `src/osm/` - OpenStreetMap PBF parser
- `src/spatial/` - R-Tree spatial index for coordinate snapping
- `src/geo/` - Geographic utilities (Haversine formula)
- `frontend/` - Leaflet.js web UI

## Code Conventions
- Follow Rust 2021 edition idioms
- Use `thiserror` for error types, `anyhow` for application errors
- All public functions must have doc comments (`///` style)
- Use `tracing` for structured logging (not `println!`)
- Prefer `impl Into<T>` and generics over concrete types in public APIs
- Error handling: Return `Result<T, E>` — never panic in library code
- Use `clippy::pedantic` lint level
- Format with `rustfmt` (default config)

## Testing
- Unit tests in the same file (`#[cfg(test)]` modules)
- Integration tests in `tests/` directory
- Benchmark tests in `benches/` using Criterion
- Test data: Use small hardcoded graphs for unit tests, never depend on large PBF files
- Run: `cargo test`, `cargo test --lib`, `cargo test --test integration`

## Key Design Decisions
- Graph stored as flat `Vec<Vec<Edge>>` adjacency list for cache locality
- Node coordinates stored in a parallel `Vec<(f64, f64)>` array
- Edge weights are `f64` representing meters (not kilometers)
- OSM node IDs (i64) mapped to compact u32 internal IDs via HashMap
- Spatial index uses `rstar` crate R-Tree for O(log N) nearest-node lookup
- API responses use GeoJSON format for path geometry

## Build & Run
- **Prerequisites**: Rust 1.70+, an `.osm.pbf` file in `data/`
- **Build**: `cargo build --release`
- **Run**: `cargo run --release -- --data data/cambodia-latest.osm.pbf`
- **Test**: `cargo test`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt --check`

## Common Pitfalls
- `f64` does not implement `Ord` in Rust; use `std::cmp::Ordering` with `partial_cmp().unwrap_or(Ordering::Equal)` or a newtype wrapper for `BinaryHeap`
- OSM ways are bidirectional by default; check `oneway=yes` tag before adding reverse edges
- PBF files contain ALL OSM data (buildings, rivers, etc.); filter by `highway` tag
- Haversine returns meters; ensure consistency with edge weights
- `BinaryHeap` in Rust is a max-heap; wrap costs in `Reverse()` or use a negated cost newtype

## Dependencies (Key Crates)
- `axum` 0.7 - Web framework
- `tokio` 1 - Async runtime
- `serde` / `serde_json` - Serialization
- `osmpbf` - OSM Protocol Buffer parser
- `rstar` - R-Tree spatial index
- `tracing` / `tracing-subscriber` - Structured logging
- `thiserror` - Error derive macros
- `anyhow` - Application error handling
- `tower-http` - CORS and middleware
- `criterion` - Benchmarking
