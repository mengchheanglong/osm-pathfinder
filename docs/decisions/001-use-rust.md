# ADR-001: Use Rust as the Implementation Language

## Status
Accepted

## Date
2026-10-07

## Context
Need a language for building a high-performance routing engine that processes million-node graphs with sub-second query times.

## Decision
Use Rust (2021 edition)

## Alternatives Considered
- Go: good concurrency but GC pauses during large graph traversals.
- C++: raw performance but memory safety risks and slower development.
- Python: too slow for graph algorithms at scale.
- Java: GC pressure with millions of small objects.

## Consequences
(+) Zero-cost abstractions, no GC pauses, memory safety guarantees, excellent ecosystem for systems programming.
(-) Steeper learning curve, longer compilation times, smaller talent pool.
