# ADR-003: Store Graph Entirely In Memory

## Status
Accepted

## Date
2026-10-07

## Context
Need to decide how to store and query the road network graph.

## Decision
Load the entire graph into RAM as Vec<Vec<Edge>> adjacency list with parallel coordinate arrays.

## Alternatives Considered
- SQLite with spatial extension: too slow for pointer-chasing graph traversal.
- Redis graph: network hop per query.
- PostgreSQL + PostGIS: production-grade but massive overhead for algorithm exploration.
- Memory-mapped files: complex, premature optimization.

## Consequences
(+) Microsecond-level node access, zero serialization overhead, CPU cache-friendly.
(-) Memory usage scales with graph size (~80MB for Cambodia), requires re-parsing PBF on every restart (until serialized cache is implemented), single-machine scaling limit.
