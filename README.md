# osm-pathfinder

![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)

> A high-performance OpenStreetMap routing engine and navigation API built with Rust.

## Overview

osm-pathfinder is a highly optimized routing engine that parses OpenStreetMap (OSM) data to build in-memory road graphs and run shortest-path algorithms. It is designed to explore graph theory algorithms with real-world geospatial data, exposing a RESTful API for integration. The engine leverages Rust's performance characteristics to achieve sub-millisecond queries on million-node graphs, zero-copy PBF parsing, and comprehensive algorithm benchmarking.

## Features

- [x] Dijkstra's shortest path algorithm
- [x] A* with Haversine great-circle heuristic
- [x] OpenStreetMap PBF file parsing
- [x] In-memory compressed graph representation
- [x] Spatial indexing (R-Tree) for coordinate snapping
- [x] RESTful API via Axum
- [x] Algorithm benchmark comparison (nodes visited, query time)
- [x] GeoJSON response format
- [ ] Bidirectional A* search
- [ ] Contraction Hierarchies preprocessing
- [ ] Turn restrictions and one-way streets
- [ ] Live traffic simulation
- [ ] Frontend map visualization (Leaflet.js)

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

4. **Test the API:**
   ```bash
   curl -X POST http://localhost:3000/api/route \
     -H "Content-Type: application/json" \
     -d '{
       "start": {"lat": 11.5564, "lon": 104.9282},
       "end": {"lat": 13.3633, "lon": 103.8564},
       "algorithm": "astar"
     }'
   ```

## API Reference

### Health Check
**Endpoint:** `GET /api/health`

Checks if the routing engine and API are operational.

### Calculate Route
**Endpoint:** `POST /api/route`

Calculates the shortest path between two coordinates.

**Request Body:**
```json
{
  "start": { "lat": 11.5564, "lon": 104.9282 },
  "end": { "lat": 13.3633, "lon": 103.8564 },
  "algorithm": "astar" // "astar" or "dijkstra"
}
```

**Response:**
```json
{
  "path_coordinates": [
    [104.9282, 11.5564],
    [104.9285, 11.5568]
  ],
  "distance_meters": 314500.5,
  "duration_seconds": 18500.2,
  "nodes_visited": 14532,
  "query_time_ms": 2.4,
  "geojson": {
    "type": "FeatureCollection",
    "features": [ ]
  }
}
```

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

## Algorithms

- **Dijkstra's Algorithm**: Guarantees the shortest path by exploring all possible routes uniformly in all directions. Excellent for baseline comparisons but can be slow over large distances.
- **A* Search (with Haversine heuristic)**: Uses the great-circle distance to the destination as a heuristic, significantly reducing the search space and execution time by prioritizing nodes that head towards the goal.

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
cargo clippy
```

## Roadmap

- Bidirectional A* search implementation
- Contraction Hierarchies (CH) for ultra-fast long-distance routing
- Turn restrictions and one-way streets support
- Live traffic simulation and dynamic weight adjustments
- Multi-modal routing support (walking, cycling, transit)
- Frontend map visualization with Leaflet.js

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Ensure that you have run tests, benchmarks, and rustfmt before submitting. 

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **OpenStreetMap Contributors** for providing the map data
- **Geofabrik** for their convenient regional OSM extracts
- The vibrant **Rust ecosystem** for providing amazing crates like Axum, Tokio, and osmpbf
