# Architecture: osm-pathfinder

This document provides a comprehensive overview of the architecture, module decomposition, and data flow for **osm-pathfinder**, a high-performance OpenStreetMap routing engine and navigation API built with Rust.

## 1. System Overview

**osm-pathfinder** is a Rust-based routing engine designed to parse OpenStreetMap (OSM) data into a highly efficient in-memory graph and serve shortest-path queries via a RESTful API.

### High-Level Data Flow

```text
OSM PBF File
    │
    ▼
┌──────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  PBF Parser  │────▶│  Graph Builder   │────▶│  Road Graph        │
│  (osm/)      │     │  (graph/builder) │     │  (graph/types)     │
└──────────────┘     └──────────────────┘     │  - Adjacency List  │
                                               │  - Coordinate Store│
                                               │  - R-Tree Index    │
                                               └────────┬───────────┘
                                                        │
                     ┌──────────────────┐               │
                     │  Axum HTTP API   │◀──────────────┘
                     │  (api/)          │
                     └───────┬──────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
       GET /health    POST /route    GET /graph/stats
                          │
                          ▼
              ┌───────────────────────┐
              │  Pathfinding Engine   │
              │  (pathfinding/)       │
              ├───────────────────────┤
              │  1. Snap coords to    │
              │     nearest nodes     │
              │  2. Run algorithm     │
              │     (Dijkstra / A*)   │
              │  3. Reconstruct path  │
              │  4. Collect metrics   │
              └───────────┬───────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │  GeoJSON Response     │
              │  + Benchmark Stats    │
              └───────────────────────┘
```

## 2. Module Decomposition

### 2.1 OSM Parser (`src/osm/`)
- Reads `.osm.pbf` files using the `osmpbf` crate.
- Processes data in two passes:
  - **Pass 1:** Collect all node coordinates referenced by highway ways.
  - **Pass 2:** Build edges from way segments, filtering by highway tags.
- **Highway tag filtering:** Includes `motorway`, `trunk`, `primary`, `secondary`, `tertiary`, `unclassified`, `residential`, and `service`.
- **Speed limit inference:** Determines speed based on highway type (e.g., motorway=120km/h, primary=80km/h, residential=30km/h, etc.).
- **One-way handling:** Respects directed edges via the `oneway` OSM tag.

### 2.2 Graph (`src/graph/`)
- `RoadGraph` struct: The central data structure holding the routing network.
  - `adjacency: Vec<Vec<Edge>>` — Adjacency list indexed by a compact internal node ID.
  - `coords: Vec<Coordinate>` — Parallel array of `(lat, lon)` for each node.
  - `osm_id_map: HashMap<i64, u32>` — Maps original OSM node IDs to compact internal IDs.
- `Edge` struct: `{ target: u32, distance_m: f64, duration_s: f64 }`.
- `Coordinate` struct: `{ lat: f64, lon: f64 }`.
- **Memory Optimization:** Uses contiguous vectors optimized for CPU cache performance.
- **Estimated memory footprint:** ~80MB for Cambodia's ~1M node road network.

### 2.3 Spatial Index (`src/spatial/`)
- Uses the `rstar` crate to build an R-Tree over all graph node coordinates.
- **Purpose:** Enables snapping arbitrary user-provided latitude/longitude pairs to the nearest valid road graph node in O(log N) time.
- **Lifecycle:** Built once at startup immediately after graph construction.

### 2.4 Pathfinding (`src/pathfinding/`)
- Provides a common trait: `PathFinder` with the method `find_path(graph, start, end) -> PathResult`.
- `PathResult`: `{ path: Vec<u32>, coordinates: Vec<Coordinate>, distance_m: f64, duration_s: f64, nodes_visited: usize, query_time: Duration }`.

#### Dijkstra
- A classic priority queue (`BinaryHeap`) based implementation.
- Uses `Reverse` wrapper to enforce min-heap behavior.
- Explores uniformly in all directions, serving as a correct baseline for comparison.
- Time complexity: O((V + E) log V).

#### A*
- Enhances Dijkstra with a heuristic priority queue: `f(n) = g(n) + h(n)`.
- **Heuristic h(n):** Uses the Haversine great-circle distance to the target node.
- **Admissible:** Since straight-line distance is always ≤ actual road distance (triangle inequality on a sphere), the heuristic guarantees the shortest path.
- Dramatically prunes the search space for long-distance queries.
- Time complexity: O((V + E) log V) worst case, but typically significantly faster in practice.

### 2.5 Geographic Utilities (`src/geo/`)
- **Haversine formula:** Calculates the great-circle distance between two geographic coordinates.
- Used both for edge weight computation (when OSM lacks explicit distance tags) and as the A* heuristic.
- Uses a standard Earth radius constant: 6,371,000 meters.

### 2.6 API Layer (`src/api/`)
- Built on Axum 0.7 utilizing the Tokio asynchronous runtime.
- **Shared state:** Maintains `Arc<AppState>` containing references to the `RoadGraph` and `SpatialIndex`.
- **CORS:** Enabled via `tower-http` to permit frontend (Leaflet.js) access.
- **Observability:** Structured logging implemented via the `tracing` crate.

## 3. Data Flow: Route Query Lifecycle

When a `POST /api/route` request is received, the following sequence occurs:
1. **Deserialization:** Axum parses the JSON request body into a `RouteRequest` struct.
2. **Snapping:** Start and end coordinates are snapped to the nearest road network nodes via the R-Tree index.
3. **Execution:** The requested algorithm (Dijkstra or A*) runs against the in-memory graph.
4. **Reconstruction:** Upon finding the target, the path is reconstructed by traversing parent pointers backward.
5. **Mapping:** Node IDs in the resulting path are mapped back to their geographic coordinates.
6. **Serialization:** The response is formatted and serialized as GeoJSON, alongside benchmarking metrics (query time, nodes visited).

## 4. Performance Characteristics

- **Graph loading time:** ~2-5 seconds for the Cambodia PBF file (~50MB).
- **Memory footprint:** ~80MB for the complete Cambodia road network.
- **Dijkstra query (Phnom Penh → Siem Reap):** ~1-3 seconds, exploring ~500K nodes.
- **A* query (same route):** ~20-50ms, exploring ~30K nodes.
- **Coordinate snapping:** <1ms per query utilizing the O(log N) R-Tree lookup.

## 5. Technology Choices & Rationale

- **Rust:** Chosen for memory safety without a garbage collector and zero-cost abstractions. Ideal for compute-heavy, pointer-chasing graph algorithms.
- **Axum:** A highly ergonomic, `tower`-native web framework maintained by the Tokio team, providing compile-time route checking and excellent async integration.
- **osmpbf:** Provides an efficient streaming parser for PBF files directly in Rust.
- **rstar:** A pure-Rust implementation of R-Trees, eliminating the need for C bindings and maintaining memory safety.
- **In-memory graph vs. Database:** Native graph algorithms require rapid, localized pointer-chasing operations. External databases introduce prohibitive per-hop I/O and network latencies.

## 6. Future Architecture Extensions

- **Bidirectional search:** Running forward and backward searches simultaneously to halve the search radius.
- **Contraction Hierarchies (CH):** Adding a preprocessing step to inject "shortcut" edges, enabling sub-millisecond continental-scale queries.
- **Graph partitioning:** Introducing spatial partitioning to support multi-threaded parallel searches.
- **Persistent graph cache:** Serializing the heavily processed in-memory graph to a binary format to bypass PBF parsing on subsequent boots.
- **WebSocket streaming:** Streaming the search wavefront progress to the frontend for real-time visualization of the algorithm execution.

## 7. Deployment Model

- **Single binary:** Rust compiles down to a self-contained native executable.
- **Containerization:** Deployed via a Docker container utilizing a multi-stage build (builder phase + slim runtime phase).
- **Data storage:** PBF files are provided via a mounted data volume.
- **Stateless API:** The graph is built entirely in memory at startup, and runtime operations are strictly read-only, making the API fully stateless and scalable.
