# ADR-004: Use OpenStreetMap as the Data Source

## Status
Accepted

## Date
2026-10-07

## Context
Need real-world road network data for routing.

## Decision
Use OpenStreetMap PBF extracts from Geofabrik.

## Alternatives Considered
- Google Maps API: cannot download raw graph data, closed API, ToS restrictions.
- HERE Maps: commercial license.
- TomTom: commercial.
- manually digitized data: not scalable.

## Consequences
(+) Free and open data, global coverage, rich road metadata (speed limits, road types, one-way), active community maintaining data quality, PBF format is compact and fast to parse.
(-) Data quality varies by region, some roads may be missing or inaccurately tagged, need to handle OSM's data model complexity (ways, nodes, relations).
