# osm-pathfinder

![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)

> A high-performance OpenStreetMap routing engine and navigation API built with Rust.

## Overview

osm-pathfinder is a highly optimized routing engine that parses OpenStreetMap (OSM) data to build in-memory road graphs and run shortest-path algorithms. It is designed to explore graph theory algorithms with real-world geospatial data, exposing a RESTful API for integration. The engine leverages Rust's performance characteristics to achieve sub-millisecond queries on million-node graphs, zero-copy PBF parsing, and comprehensive algorithm benchmarking.

## Features

- [x] Dijkstra's shortest path algorithm (uniform-cost baseline)
- [x] A* with Haversine great-circle heuristic
- [x] Bidirectional A* search (dual meeting wavefronts)
- [x] Bidirectional Dijkstra search
- [x] Contraction Hierarchies (CH) preprocessing & upward query (<1 ms latency)
- [x] Time-Dependent Shortest Path (TDSP) traffic congestion simulation
- [x] OpenStreetMap PBF file parsing (streaming two-pass)
- [x] Built-in demo road network (runs out-of-the-box without large downloads)
- [x] In-memory compressed graph representation (`Vec<Vec<Edge>>`)
- [x] Spatial indexing (R-Tree via `rstar`) for coordinate snapping
- [x] RESTful API via Axum with static file serving
- [x] Side-by-side benchmark comparison (5 algorithms, search space pruning %)
- [x] Travel-time isochrone generation (reachability polygons & GeoJSON contours)
- [x] Geographic bearing, turn angle & traffic turn penalty calculations
- [x] GeoJSON response format & interactive search wavefront visualization
- [x] Modern interactive web UI (Leaflet.js with Routing & Isochrone tabs)

## Quick Start

### Prerequisites
- Rust 1.70 or higher
- An OSM PBF file

### Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/mengchheanglong/osm-pathfinder.git
   cd osm-pathfinder
   ```

2. **Download sample data:**
   We recommend downloading a small region like Cambodia for initial testing.
   ```bash
   mkdir data
   curl -L -o data/cambodia-latest.osm.pbf https://download.geofabrik.de/asia/cambodia-latest.osm.pbf
   ```

3. **Build and Run:**
   ```bash
   export OSM_DATA_PATH=./data/cambodia-latest.osm.pbf
   cargo run --release
   ```

4. **Test the API or open the Web UI:**
   - Open your browser to `http://localhost:3000` to interact with the Leaflet map.
   - Or test via curl:
   ```bash
   curl -X POST http://localhost:3000/api/route \
     -H "Content-Type: application/json" \
     -d '{
       "start_lat": 11.5564,
       "start_lon": 104.9282,
       "end_lat": 13.3671,
       "end_lon": 103.8448,
       "algorithm": "astar"
     }'
   ```

## API Reference

### Health Check
**Endpoint:** `GET /api/health`

Checks if the routing engine and API are operational.

### Calculate Route
**Endpoint:** `POST /api/route`

Calculates the shortest or fastest path between two coordinates.

**Supported Algorithms:**
- `astar`: A* search with Haversine great-circle heuristic
- `bidirectional_astar`: Bi-directional A* with dual meeting wavefronts
- `dijkstra`: Classic Dijkstra uniform-cost search
- `bidirectional_dijkstra`: Dual-directional uniform-cost search
- `contraction_hierarchies`: Preprocessed highway hierarchies (< 1 ms latency)

**Request Body:**
```json
{
  "start_lat": 11.5564,
  "start_lon": 104.9282,
  "end_lat": 13.3671,
  "end_lon": 103.8448,
  "algorithm": "contraction_hierarchies",
  "metric": "distance",
  "departure_time": "08:15",
  "include_explored": false
}
```

**Response:**
```json
{
  "path": [
    [104.9282, 11.5564],
    [104.9285, 11.5568]
  ],
  "distance_m": 314500.5,
  "duration_s": 14200.2,
  "nodes_visited": 42,
  "query_time_ms": 0.45,
  "algorithm": "contraction_hierarchies",
  "start_snapped": [104.9282, 11.5564],
  "end_snapped": [103.8448, 13.3671],
  "explored": []
}
```

### Reachability Isochrones
**Endpoint:** `POST /api/isochrone` or `GET /api/isochrone?lat=...&lon=...&buckets=10,20,30`

Computes concentric travel-time reachability boundary polygons (GeoJSON FeatureCollection) from an origin point.

**Request Body:**
```json
{
  "lat": 11.5564,
  "lon": 104.9282,
  "buckets": [10, 20, 30, 45],
  "departure_time": "08:15"
}
```

**Response:**
Standard GeoJSON `FeatureCollection` with `Polygon` geometries, surface area in $\text{km}^2$, node counts, and visualization styling for Leaflet.js.

### Graph Statistics
**Endpoint:** `GET /api/graph/stats`

Returns statistics about the loaded road network graph (number of nodes, edges, etc.).

## Architecture Overview

The system architecture is designed for speed and efficiency, from the initial parsing of OSM data to serving HTTP requests. For more details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

```text
.osm.pbf file → PBF Parser → Road Graph (in-memory) → Pathfinding Engine
                                                                 ↓
                                           Axum HTTP API ← Route Calculator
                                                 ↓
                                           JSON/GeoJSON Response → Frontend Map
```

## Project Structure

```text
osm-pathfinder/
├── src/
│   ├── main.rs              # Application entry point & server startup
│   ├── api/                 # HTTP handlers and routing
│   │   ├── mod.rs
│   │   ├── routes.rs        # Route definitions
│   │   └── handlers.rs      # Request handlers
│   ├── graph/               # Graph data structures
│   │   ├── mod.rs
│   │   ├── builder.rs       # Graph construction from OSM data
│   │   └── types.rs         # Node, Edge, Graph types
│   ├── pathfinding/         # Shortest path algorithms
│   │   ├── mod.rs
│   │   ├── dijkstra.rs      # Dijkstra's algorithm
│   │   ├── astar.rs         # A* with Haversine heuristic
│   │   └── types.rs         # PathResult, BenchmarkStats
│   ├── osm/                 # OpenStreetMap data parsing
│   │   ├── mod.rs
│   │   └── parser.rs        # PBF file parser
│   ├── spatial/             # Spatial indexing
│   │   ├── mod.rs
│   │   └── index.rs         # R-Tree for coordinate snapping
│   └── geo/                 # Geographic utilities
│       ├── mod.rs
│       └── haversine.rs     # Haversine distance formula
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
├── data/                    # OSM data files (gitignored)
├── docs/                    # Documentation
│   ├── ARCHITECTURE.md
│   └── decisions/           # Architecture Decision Records
├── frontend/                # Leaflet.js map UI
├── Cargo.toml
├── AGENTS.md
├── LICENSE
└── README.md
```

## Algorithms & Theoretical Foundations

1. **Dijkstra's Algorithm (Uniform Cost)**:
   - Computes guaranteed shortest path by exploring nodes in ascending order of cost $g(u)$.
   - Serves as the mathematical baseline for correctness and search space comparison.

2. **A* Search (Haversine Heuristic)**:
   - Evaluates $f(u) = g(u) + h(u)$ using the great-circle Haversine formula as $h(u)$.
   - Admissible ($h(u) \le d^*(u)$) and monotonic, pruning 70–90% of the search space compared to Dijkstra while preserving mathematical optimality.

3. **Bi-directional Dijkstra**:
   - Launches simultaneous forward and reverse wavefronts meeting at the midpoint.
   - Reduces search area by up to 50% ($2 \cdot \pi (r/2)^2 = \frac{1}{2} \pi r^2$).

4. **Bi-directional A* Search**:
   - Combines balanced symmetric potentials with simultaneous dual-directional exploration.

5. **Contraction Hierarchies (CH)**:
   - Two-phase technique: preprocessing contracts nodes by importance and inserts shortcut edges; queries execute upward bidirectional Dijkstra on shortcut graphs.
   - Yields queries in **< 1 ms** even on nationwide networks.

6. **Time-Dependent Shortest Path (TDSP)**:
   - Simulates realistic urban congestion using temporal Gaussian peak distributions (morning 08:15 AM & evening 17:45 PM rush hours) coupled with radial geographic decay around city centers.

## Configuration

The application can be configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `OSM_DATA_PATH` | Path to the OSM PBF file to load | `./data/map.osm.pbf` |
| `SERVER_HOST` | Host address to bind the API server | `127.0.0.1` |
| `SERVER_PORT` | Port for the API server | `3000` |
| `LOG_LEVEL` | Logging verbosity (debug, info, warn, error) | `info` |

## Development

Run tests:
```bash
cargo test
```

Run benchmarks:
```bash
cargo bench
```

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy -- -D warnings
```

## Roadmap

- [x] Travel-time isochrone generation (reachability polygons)
- [x] Geographic bearing, turn angle & turn penalty estimation
- [ ] Turn restrictions from OSM relation data (`type=restriction`)
- [ ] Multi-modal routing profiles (car, bicycle, pedestrian speeds & access)
- [ ] Dynamic real-time GTFS / transit schedule integration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Ensure that you have run tests, benchmarks, and rustfmt before submitting. 

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **OpenStreetMap Contributors** for providing the map data
- **Geofabrik** for their convenient regional OSM extracts
- The vibrant **Rust ecosystem** for providing amazing crates like Axum, Tokio, and osmpbf
