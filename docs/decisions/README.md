# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) documenting significant technical decisions made in the osm-pathfinder project.

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-use-rust.md) | Use Rust as the Implementation Language | Accepted | 2026-10-07 |
| [002](002-axum-web-framework.md) | Use Axum as the Web Framework | Accepted | 2026-10-07 |
| [003](003-in-memory-graph.md) | Store Graph Entirely In Memory | Accepted | 2026-10-07 |
| [004](004-osm-data-source.md) | Use OpenStreetMap as the Data Source | Accepted | 2026-10-07 |
| [005](005-algorithm-selection.md) | Implement Dijkstra and A* as Core Algorithms | Accepted | 2026-10-07 |

## Process
When making significant technical decisions, create a new ADR following the template above. Number sequentially. ADRs are immutable once accepted — if a decision is reversed, create a new ADR referencing the old one.
