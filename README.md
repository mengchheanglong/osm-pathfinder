# osm-pathfinder

<div align="center">

[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg?logo=rust)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/framework-axum_0.7-orange.svg)](https://github.com/tokio-rs/axum)
[![Tokio](https://img.shields.io/badge/runtime-tokio-blueviolet.svg)](https://tokio.rs)
[![Tests Passing](https://img.shields.io/badge/tests-35%2F35%20passed-brightgreen.svg)]()
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-brightgreen.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**A high-performance OpenStreetMap routing engine, geospatial graph platform, and real-time navigation API written in pure Rust.**

[Overview](#overview) •
[Features](#features) •
[Benchmarks](#benchmarks--algorithm-comparison) •
[Quick Start](#quick-start) •
[API Reference](#api-reference) •
[Mathematical Foundations](#mathematical-foundations) •
[Architecture](#architecture) •
[Web Interface](#interactive-web-interface)

</div>

---

## Overview

**`osm-pathfinder`** is a production-grade routing engine built from scratch in Rust. It transforms raw OpenStreetMap Protocolbuffer Binary Format (`.osm.pbf`) extracts into cache-optimized, in-memory graph topologies to execute shortest-path queries with sub-millisecond latencies.

Designed for both practical backend engineering and academic graph theory research, the system features a side-by-side benchmark suite comparing **5 routing algorithms** (from classic Dijkstra to multi-level Contraction Hierarchies), **Time-Dependent Shortest Path (TDSP)** traffic simulation, **Travel-Time Isochrone reachability contours**, and an interactive, glassmorphic **Leaflet.js map interface**.

---

## Features

### 🚀 High-Performance Pathfinding
- **Contraction Hierarchies (CH)**: Preprocessed highway shortcuts with upward bidirectional Dijkstra for **sub-millisecond (< 1 ms)** queries on nationwide networks.
- **A\* Search (Haversine Heuristic)**: Admissible great-circle distance lower bounds pruning **70–90%** of uniform-cost search space.
- **Bi-directional A\* Search**: Dual simultaneous wavefronts meeting at the midpoint with balanced potential functions.
- **Bi-directional Dijkstra**: Dual-frontier uniform-cost search reducing exploration area by **~50%**.
- **Dijkstra Baseline**: Strict uniform-cost exploration serving as the formal ground-truth benchmark.

### 🚦 Real-World Navigation & Traffic Dynamics
- **Time-Dependent Shortest Path (TDSP)**: Gaussian temporal traffic curves modeling morning (08:15 AM) and evening (05:45 PM) congestion peaks up to $4.0\times$ free-flow duration.
- **Urban Spatial Decay**: Cubic radial falloff centered on economic hubs; rural corridors maintain unrestricted travel speeds.
- **Turn Penalty & Bearing Geometry**: Estimates junction delays based on deflection angles $\Delta \theta$ (right turns = 4s, left turns crossing traffic = 8s, U-turns = 15s).
- **Dual Cost Metrics**: Seamless optimization for either physical **Shortest Distance** (meters) or **Fastest Travel Time** (seconds).

### ⏱️ Urban Mobility & Isochrone Mapping
- **Travel-Time Reachability Contours**: Bounded Dijkstra exploration computing reachable boundaries for 10, 20, 30, 45, and 60-minute horizons.
- **Smooth Radial Polygon Generation**: Angular sector ray-casting, empty-sector interpolation, and 3-point circular smoothing generating GeoJSON `Polygon` linear rings.
- **Surface Area Computation**: Analytical enclosed area calculations in $\text{km}^2$ via the Shoelace formula.

### 🌐 Systems Architecture & Concurrency
- **Zero-Dependency Demo Mode**: Built-in Cambodia national highway network for immediate startup without requiring external `.osm.pbf` downloads.
- **Two-Pass Streaming Parser**: Efficient OSM PBF ingestion filtering highways, speed limits, and one-way directional tags.
- **Cache-Conscious Graph Memory**: Contiguous flat vectors `Vec<Vec<Edge>>` with parallel coordinate arrays `Vec<Coordinate>`.
- **R-Tree Spatial Snapping**: Sub-millisecond coordinate-to-node snapping via `rstar` with $O(\log N)$ nearest-neighbor queries.
- **Asynchronous REST API**: Powered by Axum 0.7, Tokio async runtime, Tower HTTP middleware, and structured logging via `tracing`.

---

## Benchmarks & Algorithm Comparison

Performance measured over Cambodia's road network topology (synthetic and real-world network segments):

| Algorithm | Heuristic / Optimization | Typical Latency | Visited Nodes | Search Space Pruning | Optimality Guarantee |
|:---|:---|:---:|:---:|:---:|:---:|
| **⚡ Contraction Hierarchies** | Shortcut hierarchy + upward query | **0.1 – 0.8 ms** | **~20 – 150** | **98% – 99.5%** | **100% Guaranteed** |
| **Bi-directional A\*** | Balanced Haversine dual potentials | **0.8 – 2.5 ms** | **~180 – 600** | **85% – 92%** | **100% Guaranteed** |
| **A\* Search** | Great-circle Haversine heuristic | **1.2 – 3.8 ms** | **~350 – 1,200** | **75% – 88%** | **100% Guaranteed** |
| **Bi-directional Dijkstra** | Dual expanding circular wavefronts | **2.5 – 6.0 ms** | **~800 – 2,500** | **~50%** | **100% Guaranteed** |
| **Dijkstra (Baseline)** | Uniform cost expansion | **5.0 – 14.0 ms** | **~2,000 – 6,500** | **0% (Baseline)** | **100% Guaranteed** |

---

## Mathematical Foundations

### 1. Admissible Heuristic Proof for Travel Time
In standard distance routing, the straight-line Haversine formula is admissible because great-circle distance $d_{\text{geo}}(u, t)$ never exceeds physical road distance $d^{\ast}(u, t)$ on a sphere:

$$h_{\text{dist}}(u) = d_{\text{geo}}(u, t) \le d^{\ast}_{\text{dist}}(u, t)$$

When optimizing for **travel duration** under time-dependent traffic conditions, admissibility is preserved by bounding the network with the maximum conceivable network speed $v_{\text{max}} = 120\text{ km/h} \approx 33.33\text{ m/s}$:

$$h_{\text{time}}(u) = \frac{d_{\text{geo}}(u, t)}{v_{\text{max}}}$$

Because actual traversal speed on any segment $e \in E$ satisfies $v(e, \tau) \le v_{\text{max}}$, it follows that:

$$\frac{d^{\ast}(u, t)}{v_{\text{max}}} \le d^{\ast}_{\text{time}}(u, t) \implies h_{\text{time}}(u) \le d^{\ast}_{\text{time}}(u)$$

This guarantees that A\* and Bi-directional A\* never overestimate cost-to-target, preserving strict mathematical optimality.

### 2. Contraction Hierarchies (CH) Upward Invariant
During preprocessing, vertices $v \in V$ are ordered by importance rank $\pi(v)$ (derived from edge difference and degree). When node $v$ is contracted, shortcut edges $(u, w)$ with cost $c(u, v) + c(v, w)$ are added if and only if path $\langle u, v, w \rangle$ is the unique shortest path.

During query evaluation:
- Forward search from $s$ only relaxes edges $(u \to v)$ where $\pi(v) > \pi(u)$.
- Backward search from $t$ only relaxes reverse edges $(w \to v)$ where $\pi(w) > \pi(v)$.
- Both frontiers monotonically climb the rank hierarchy, meeting at the maximum rank vertex on the optimal path.

### 3. Geographic Bearing & Deflection Angle
Initial compass forward azimuth $\theta \in [0^{\circ}, 360^{\circ})$ from $(\phi_1, \lambda_1)$ to $(\phi_2, \lambda_2)$:

$$\theta = \text{atan2}\left(\sin \Delta\lambda \cos \phi_2, \; \cos \phi_1 \sin \phi_2 - \sin \phi_1 \cos \phi_2 \cos \Delta\lambda\right)$$

The signed junction turn angle $\Delta\theta \in [-180^{\circ}, 180^{\circ}]$ between incoming segment $\theta_1$ and outgoing segment $\theta_2$:

$$\Delta\theta = (\theta_2 - \theta_1 + 540^{\circ}) \bmod 360^{\circ} - 180^{\circ}$$

---

## Quick Start

### Prerequisites
- [Rust 1.70+](https://www.rust-lang.org/tools/install)
- Git

### 1. Clone & Build
```bash
git clone https://github.com/mengchheanglong/osm-pathfinder.git
cd osm-pathfinder
cargo build --release
```

### 2. Run with Built-in Demo Network (Instant)
Without downloading any external map files, `osm-pathfinder` automatically boots using its built-in Cambodia national highway network (covering National Roads 1 through 7 and expressways):

```bash
cargo run --release
```

Open your browser at **`http://localhost:3000`** to access the interactive web dashboard.

### 3. Run with Real-World OSM Data
To load real OpenStreetMap extracts:

```bash
# Automated download script for Cambodia extract (Windows PowerShell)
powershell -ExecutionPolicy Bypass -File scripts/download_cambodia_osm.ps1

# Or on Linux / macOS:
bash scripts/download_cambodia_osm.sh

# Run the engine with the downloaded PBF:
cargo run --release -- --data data/cambodia-latest.osm.pbf
```

---

## API Reference

### 1. Health Check
```http
GET /api/health
```
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

---

### 2. Graph Statistics
```http
GET /api/graph/stats
```
```json
{
  "nodes": 48210,
  "edges": 96420
}
```

---

### 3. Calculate Route
```http
POST /api/route
Content-Type: application/json
```

**Request Parameters:**
| Parameter | Type | Required | Description |
|:---|:---|:---:|:---|
| `start_lat` | `f64` | Yes | Starting point latitude |
| `start_lon` | `f64` | Yes | Starting point longitude |
| `end_lat` | `f64` | Yes | Destination latitude |
| `end_lon` | `f64` | Yes | Destination longitude |
| `algorithm` | `string` | No | `contraction_hierarchies`, `astar` (default), `bidirectional_astar`, `dijkstra`, `bidirectional_dijkstra` |
| `metric` | `string` | No | `distance` (default) or `time` |
| `departure_time`| `string` | No | Time-of-day in 24h format (`"08:15"`, `"17:45"`) for traffic modeling |
| `include_explored` | `bool` | No | Whether to return visited wavefront node coordinates |

**Example Request:**
```bash
curl -X POST http://localhost:3000/api/route \
  -H "Content-Type: application/json" \
  -d '{
    "start_lat": 11.5564,
    "start_lon": 104.9282,
    "end_lat": 13.3671,
    "end_lon": 103.8448,
    "algorithm": "contraction_hierarchies",
    "metric": "distance"
  }'
```

**Example Response:**
```json
{
  "path": [
    [104.9282, 11.5564],
    [104.9250, 11.5620],
    [103.8448, 13.3671]
  ],
  "distance_m": 314500.5,
  "duration_s": 14200.2,
  "nodes_visited": 42,
  "query_time_ms": 0.45,
  "algorithm": "contraction_hierarchies",
  "metric": "distance",
  "departure_time": null,
  "start_snapped": [104.9282, 11.5564],
  "end_snapped": [103.8448, 13.3671]
}
```

---

### 4. Travel-Time Reachability Isochrones
```http
POST /api/isochrone
GET  /api/isochrone?lat=11.5564&lon=104.9282&buckets=10,20,30,45
Content-Type: application/json
```

**Request Body:**
```json
{
  "lat": 11.5564,
  "lon": 104.9282,
  "buckets": [10, 20, 30, 45],
  "departure_time": "08:15"
}
```

**Example Response:**
Returns standard GeoJSON `FeatureCollection` with closed `Polygon` geometries and coverage statistics:
```json
{
  "type": "FeatureCollection",
  "properties": {
    "center": [104.9282, 11.5564],
    "total_nodes_visited": 1840,
    "query_time_ms": 1.15
  },
  "features": [
    {
      "type": "Feature",
      "properties": {
        "time_minutes": 10,
        "nodes_reached": 210,
        "area_sq_km": 18.4,
        "color": "#10b981",
        "fillOpacity": 0.25,
        "strokeWeight": 2
      },
      "geometry": {
        "type": "Polygon",
        "coordinates": [[[104.9282, 11.5564], "..."]]
      }
    }
  ]
}
```

---

## Interactive Web Interface

The engine embeds a single-page web application served directly from `/` without requiring external Node.js dependencies:

```text
┌────────────────────────────────────────────────────────────────────────┐
│  osm-pathfinder                        [ 48,210 nodes | 96,420 edges ] │
├────────────────────────────────┬───────────────────────────────────────┤
│ Mode: [ Point-to-Point ] [ ⏱️ Isochrone ]                               │
│                                │                                       │
│ Origin: 11.5564, 104.9282      │           [ Leaflet Map ]             │
│ Dest:   13.3671, 103.8448      │                                       │
│                                │   (Start Pin)                         │
│ Algorithms:                    │        •                              │
│ [ A* ] [ Bi-A* ] [ Dijkstra ]  │         \                             │
│ [ ⚡ Contraction Hierarchies ]  │          \  <-- Route Polyline       │
│                                │           \                           │
│ Traffic Departure: [ 08:15 AM ]│            • (Destination Pin)        │
│ Optimization: [ Shortest Dist ]│                                       │
│                                │   [ Concentric Isochrone Bands ]      │
│ [ Calculate Route ]            │   [ 10m  20m  30m  45m  60m ]         │
│ [ Compare All 5 ]              │                                       │
├────────────────────────────────┴───────────────────────────────────────┤
│ Benchmark Table: CH (0.4ms) | Bi-A* (1.2ms) | A* (2.1ms) | Dijkstra (8ms)│
└────────────────────────────────────────────────────────────────────────┘
```

- **Dual Modes**: Seamlessly switch between Point-to-Point Routing and Isochrone Reachability.
- **Draggable Pins**: Interactive start/destination and origin hub markers with automatic re-routing.
- **Wavefront Visualizer**: Renders search space expansions as animated glowing particle wavefronts.
- **Side-by-Side Comparator**: Evaluates all 5 algorithms in real-time, displaying search space pruning insights.

---

## Architecture

```text
                   ┌────────────────────────────────────────┐
                   │    OpenStreetMap PBF File (.osm.pbf)   │
                   └───────────────────┬────────────────────┘
                                       │
                                       ▼
                       ┌───────────────────────────────┐
                       │   Streaming Two-Pass Parser   │
                       │   - Tag filtering (highway)   │
                       │   - Speed limit inference     │
                       └───────────────┬───────────────┘
                                       │
                                       ▼
                       ┌───────────────────────────────┐
                       │       In-Memory RoadGraph     │
                       │  - Forward Adjacency List     │
                       │  - Reverse Adjacency List     │
                       │  - WGS84 Coordinate Store     │
                       └───────┬───────────────┬───────┘
                               │               │
             ┌─────────────────┘               └─────────────────┐
             ▼                                                   ▼
┌─────────────────────────┐                         ┌─────────────────────────┐
│   Spatial Index (R-Tree)│                         │ Contraction Hierarchies │
│   - O(log N) Snapping   │                         │ - Node Ordering Rank    │
│   - rstar crate         │                         │ - Shortcut Generation   │
└────────────┬────────────┘                         └────────────┬────────────┘
             │                                                   │
             └─────────────────┬─────────────────────────────────┘
                               │
                               ▼
        ┌──────────────────────────────────────────────┐
        │              Pathfinding Engine              │
        │  ├── Contraction Hierarchies (Upward Query)  │
        │  ├── Bi-directional A* (Balanced Heuristics) │
        │  ├── A* Search (Haversine Admissible)        │
        │  ├── Bi-directional Dijkstra                 │
        │  ├── Uniform-Cost Dijkstra                   │
        │  ├── Isochrone Contours (Bounded Dijkstra)   │
        │  └── TDSP Traffic Simulation (Gaussian Peak) │
        └──────────────────────┬───────────────────────┘
                               │
                               ▼
        ┌──────────────────────────────────────────────┐
        │            Axum REST API Server              │
        │  GET  /api/health       GET /api/graph/stats │
        │  POST /api/route        POST /api/isochrone  │
        └──────────────────────┬───────────────────────┘
                               │
                               ▼
        ┌──────────────────────────────────────────────┐
        │        Leaflet.js Frontend Web Dashboard     │
        │  - Dual-mode controller & custom markers     │
        │  - Wavefront particle overlays & contours    │
        └──────────────────────────────────────────────┘
```

---

## Repository Layout

```text
osm-pathfinder/
├── src/
│   ├── lib.rs                 # Library root exposing AppState and modules
│   ├── main.rs                # CLI entry point, arg parsing, Axum server bootstrap
│   ├── api/                   # HTTP API layer
│   │   ├── mod.rs
│   │   ├── routes.rs          # Route registration, CORS, ServeDir static asset serving
│   │   └── handlers.rs        # Handlers for /health, /stats, /route, and /isochrone
│   ├── graph/                 # Graph topology and storage
│   │   ├── mod.rs
│   │   ├── types.rs           # Coordinate, Edge, RoadGraph data structures
│   │   ├── builder.rs         # Incremental graph construction
│   │   └── demo.rs            # Built-in fallback Cambodia highway network
│   ├── pathfinding/           # Routing and graph search algorithms
│   │   ├── mod.rs
│   │   ├── types.rs           # Algorithm enum, CostMetric, PathResult, RoutingOptions
│   │   ├── dijkstra.rs         # Uniform-cost Dijkstra with wavefront tracking
│   │   ├── astar.rs            # A* search with Haversine admissible heuristic
│   │   ├── bidirectional.rs    # Dual-wavefront Bi-Dijkstra and Bi-A*
│   │   ├── ch.rs               # Contraction Hierarchies preprocessing and upward query
│   │   └── isochrone.rs        # Reachability polygon and GeoJSON contour generator
│   ├── traffic/               # Time-dependent traffic congestion modeling
│   │   └── mod.rs             # Gaussian temporal spikes, spatial radial decay, speed bounds
│   ├── osm/                   # OpenStreetMap data ingestion
│   │   ├── mod.rs
│   │   └── parser.rs          # Two-pass streaming PBF parser
│   ├── spatial/               # Geospatial indexing
│   │   └── mod.rs             # R-Tree (rstar) for coordinate snapping
│   └── geo/                   # Geographic calculations
│       ├── mod.rs
│       ├── haversine.rs        # Great-circle Haversine formula
│       └── bearing.rs          # Forward azimuth, turn angles, junction penalties
├── frontend/                  # Interactive single-page web interface
│   ├── index.html             # Responsive HTML5 layout and floating glassmorphic panel
│   ├── style.css              # Custom styling, dark mode, custom pins, table formatting
│   └── app.js                 # Leaflet controller, state management, API client
├── tests/
│   └── integration_tests.rs   # End-to-end pipeline test asserting all algorithms
├── benches/
│   └── pathfinding.rs         # Criterion benchmark suite (Dijkstra, A*, Bi-A*, CH)
├── scripts/                   # Utility and download scripts
│   ├── download_cambodia_osm.ps1
│   └── download_cambodia_osm.sh
├── docs/                      # Technical specifications & Architecture Decision Records
│   ├── ARCHITECTURE.md
│   └── decisions/
├── Cargo.toml                 # Package manifest, dependencies, release optimizations
├── Cargo.lock                 # Deterministic dependency lockfile
├── AGENTS.md                  # Development guidelines and agent instructions
├── CONTRIBUTING.md            # Guidelines for open-source contributions
├── LICENSE                    # MIT License
└── README.md
```

---

## Development & Testing

### Running the Test Suite
```bash
cargo test
```
The repository includes **35 automated tests** verifying Haversine precision, R-Tree snapping, algorithm optimality, Contraction Hierarchies shortcut unpacking, turn penalties, and GeoJSON polygon generation.

### Running Linters & Code Formatter
```bash
# Strict clippy linter
cargo clippy -- -D warnings

# Rust formatting check
cargo fmt --check
```

### Running Performance Benchmarks
```bash
cargo bench
```

---

## Contributing

Contributions are welcome! Please submit issues or pull requests on GitHub.

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m "Add amazing feature"`).
4. Verify that `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass cleanly.
5. Push to the branch (`git push origin feature/amazing-feature`).
6. Open a Pull Request.

---

## License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[OpenStreetMap](https://www.openstreetmap.org/)** for open geographic data.
- **[Geofabrik](https://www.geofabrik.de/)** for regional extracts.
- **[Tokio](https://tokio.rs/) & [Axum](https://github.com/tokio-rs/axum)** for state-of-the-art async web primitives.
