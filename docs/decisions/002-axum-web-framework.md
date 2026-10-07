# ADR-002: Use Axum as the Web Framework

## Status
Accepted

## Date
2026-10-07

## Context
Need an async HTTP framework for serving routing queries.

## Decision
Use Axum 0.7

## Alternatives Considered
- Actix-web: mature but uses its own actor system, heavier abstraction.
- Rocket: convenient macros but slower adoption of async.
- Warp: filter-based composition is harder to read for newcomers.
- Poem: newer, smaller community.

## Consequences
(+) Native Tokio/Tower integration, compile-time route checking, strong typing, active maintenance by Tokio team.
(-) Relatively newer than Actix-web, fewer tutorials available.
