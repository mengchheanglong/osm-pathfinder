# ADR-005: Implement Dijkstra and A* as Core Algorithms

## Status
Accepted

## Date
2026-10-07

## Context
Need to select which shortest-path algorithms to implement for the routing engine.

## Decision
Implement both Dijkstra's algorithm and A* search with Haversine heuristic as the initial algorithm set.

## Alternatives Considered
- Bellman-Ford: handles negative weights but O(VE) too slow for large graphs.
- Floyd-Warshall: O(V³) all-pairs, infeasible for >10K nodes.
- only Dijkstra: misses the opportunity to demonstrate heuristic pruning.
- only A*: loses the baseline comparison.
- Contraction Hierarchies: excellent query performance but complex preprocessing, planned for future.

## Consequences
(+) Dijkstra provides correctness baseline, A* demonstrates dramatic speedup via heuristic, side-by-side comparison creates compelling math assignment data, both are well-understood with provable optimality guarantees.
(-) Neither is fast enough for continent-scale queries without preprocessing (addressed by future CH implementation).
