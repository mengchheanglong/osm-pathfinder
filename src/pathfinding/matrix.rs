//! Directed cost matrix calculation with vehicle profiles and time-dependent traffic.
//!
//! Computes asymmetric N x M matrices of travel durations and distances,
//! strictly respecting vehicle road class gating, speed caps, and turn penalties.
//! Unreachable pairs populate with `None` (`null` in JSON) — never falling back
//! to synthetic or straight-line estimations.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use serde::Serialize;

use crate::geo::bearing::{initial_bearing, turn_angle};
use crate::geo::haversine;
use crate::graph::{Coordinate, RoadGraph};
use crate::pathfinding::{CostMetric, VehicleProfile};
use crate::spatial::SpatialIndex;
use crate::traffic;

/// Response body for the directed cost matrix endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct MatrixResponse {
    /// N x M travel durations in seconds (null if unreachable).
    pub durations: Vec<Vec<Option<f64>>>,
    /// N x M travel distances in meters (null if unreachable).
    pub distances: Vec<Vec<Option<f64>>>,
    /// Snapped coordinates of origins `[[lon, lat], ...]`.
    pub origins_snapped: Vec<[f64; 2]>,
    /// Snapped coordinates of destinations `[[lon, lat], ...]`.
    pub destinations_snapped: Vec<[f64; 2]>,
    /// Pinned graph version identifier.
    pub graph_version: String,
    /// Pinned cost model version identifier.
    pub cost_model_version: String,
}

/// Error returned during matrix calculation.
#[derive(Debug, thiserror::Error)]
pub enum MatrixError {
    #[error("Coordinate outside routable network bounds")]
    OutOfBounds,
    #[error("No graph node found near coordinate")]
    Unsnappable,
}

#[derive(Debug, Clone, Copy)]
struct QueueEntry {
    node: u32,
    cost: f64,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for QueueEntry {}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

/// Validates that a coordinate is within the Cambodia bounding box: lat [10.0, 15.0], lon [102.0, 108.0].
pub fn validate_cambodia_bounds(lat: f64, lon: f64) -> bool {
    (10.0..=15.0).contains(&lat) && (102.0..=108.0).contains(&lon)
}

/// Computes an N x M directed travel duration and distance matrix.
pub fn compute_cost_matrix(
    graph: &RoadGraph,
    spatial_index: &SpatialIndex,
    origins: &[Coordinate],
    destinations: &[Coordinate],
    profile: VehicleProfile,
    metric: CostMetric,
    departure_time: Option<&str>,
) -> Result<MatrixResponse, MatrixError> {
    // 1. Validate geographic bounding box for all coordinates
    for coord in origins.iter().chain(destinations.iter()) {
        if !validate_cambodia_bounds(coord.lat, coord.lon) {
            return Err(MatrixError::OutOfBounds);
        }
    }

    // 2. Snap origins to road network
    let mut origins_snapped = Vec::with_capacity(origins.len());
    let mut origin_nodes = Vec::with_capacity(origins.len());
    for orig in origins {
        let (node, lat, lon) = spatial_index
            .nearest_node(orig.lat, orig.lon)
            .ok_or(MatrixError::Unsnappable)?;
        let snap_dist = haversine::distance(orig.lat, orig.lon, lat, lon);
        if snap_dist > 50_000.0 {
            return Err(MatrixError::Unsnappable);
        }
        origins_snapped.push([lon, lat]);
        origin_nodes.push(node);
    }

    // 3. Snap destinations to road network
    let mut destinations_snapped = Vec::with_capacity(destinations.len());
    let mut dest_nodes = Vec::with_capacity(destinations.len());
    for dest in destinations {
        let (node, lat, lon) = spatial_index
            .nearest_node(dest.lat, dest.lon)
            .ok_or(MatrixError::Unsnappable)?;
        let snap_dist = haversine::distance(dest.lat, dest.lon, lat, lon);
        if snap_dist > 50_000.0 {
            return Err(MatrixError::Unsnappable);
        }
        destinations_snapped.push([lon, lat]);
        dest_nodes.push(node);
    }

    let dep_minutes_start = departure_time.and_then(traffic::parse_time_of_day);
    let num_nodes = graph.node_count();

    let mut durations_matrix = Vec::with_capacity(origins.len());
    let mut distances_matrix = Vec::with_capacity(origins.len());

    let target_set: HashSet<u32> = dest_nodes.iter().copied().collect();

    // 4. Compute 1-to-many Dijkstra for each origin
    for (i, &orig_node) in origin_nodes.iter().enumerate() {
        let orig_coord = origins[i];

        let mut cost_arr = vec![f64::INFINITY; num_nodes];
        let mut dist_arr = vec![f64::INFINITY; num_nodes];
        let mut dur_arr = vec![f64::INFINITY; num_nodes];
        let mut parent_arr = vec![None; num_nodes];
        let mut visited = vec![false; num_nodes];

        cost_arr[orig_node as usize] = 0.0;
        dist_arr[orig_node as usize] = 0.0;
        dur_arr[orig_node as usize] = 0.0;

        let mut heap = BinaryHeap::new();
        heap.push(QueueEntry {
            node: orig_node,
            cost: 0.0,
        });

        let mut remaining_targets = target_set.clone();

        while let Some(QueueEntry { node, cost }) = heap.pop() {
            if visited[node as usize] {
                continue;
            }
            visited[node as usize] = true;

            if remaining_targets.remove(&node) && remaining_targets.is_empty() {
                // All targets reached!
                break;
            }

            if cost > cost_arr[node as usize] {
                continue;
            }

            for edge in graph.neighbors(node) {
                // Road class permission check for vehicle profile
                if !profile.is_road_allowed(edge.road_class) {
                    continue;
                }

                let target_coord = match graph.get_coord(edge.target) {
                    Some(c) => c,
                    None => continue,
                };

                // Compute effective speed clamped to profile speed cap
                let base_speed_kmh = if edge.duration_s > 0.0 {
                    (edge.distance_m / edge.duration_s) * 3.6
                } else {
                    50.0
                };
                let effective_speed_kmh = profile.effective_speed_kmh(edge.road_class, base_speed_kmh);
                let base_duration_s = edge.distance_m / (effective_speed_kmh / 3.6);

                // Time-dependent traffic congestion factor
                let edge_dep_minutes = dep_minutes_start.map(|dep| {
                    (dep + (dur_arr[node as usize] / 60.0).floor() as u32) % 1440
                });
                let traffic_factor = traffic::congestion_multiplier(target_coord, edge_dep_minutes);
                let mut edge_duration = base_duration_s * traffic_factor;

                // Turn penalty
                if let Some(prev) = parent_arr[node as usize] {
                    if let (Some(prev_coord), Some(curr_coord)) = (graph.get_coord(prev), graph.get_coord(node)) {
                        let bearing_in = initial_bearing(prev_coord.lat, prev_coord.lon, curr_coord.lat, curr_coord.lon);
                        let bearing_out = initial_bearing(curr_coord.lat, curr_coord.lon, target_coord.lat, target_coord.lon);
                        let angle = turn_angle(bearing_in, bearing_out);
                        edge_duration += profile.turn_penalty_seconds(angle);
                    }
                }

                let edge_cost = match metric {
                    CostMetric::Distance => edge.distance_m,
                    CostMetric::Time => edge_duration,
                };

                let new_cost = cost_arr[node as usize] + edge_cost;
                if new_cost < cost_arr[edge.target as usize] {
                    cost_arr[edge.target as usize] = new_cost;
                    dist_arr[edge.target as usize] = dist_arr[node as usize] + edge.distance_m;
                    dur_arr[edge.target as usize] = dur_arr[node as usize] + edge_duration;
                    parent_arr[edge.target as usize] = Some(node);

                    heap.push(QueueEntry {
                        node: edge.target,
                        cost: new_cost,
                    });
                }
            }
        }

        // Fill row results
        let mut row_durations = Vec::with_capacity(destinations.len());
        let mut row_distances = Vec::with_capacity(destinations.len());

        for (j, &d_node) in dest_nodes.iter().enumerate() {
            let dest_coord = destinations[j];
            // Point identity: zero distance & duration if identical origin and destination
            if (orig_coord.lat - dest_coord.lat).abs() < 1e-7 && (orig_coord.lon - dest_coord.lon).abs() < 1e-7 {
                row_durations.push(Some(0.0));
                row_distances.push(Some(0.0));
            } else if visited[d_node as usize] && cost_arr[d_node as usize] < f64::INFINITY {
                row_durations.push(Some((dur_arr[d_node as usize] * 10.0).round() / 10.0));
                row_distances.push(Some((dist_arr[d_node as usize] * 10.0).round() / 10.0));
            } else {
                // Explicit null for disconnected components (island, no route, road class banned)
                row_durations.push(None);
                row_distances.push(None);
            }
        }

        durations_matrix.push(row_durations);
        distances_matrix.push(row_distances);
    }

    Ok(MatrixResponse {
        durations: durations_matrix,
        distances: distances_matrix,
        origins_snapped,
        destinations_snapped,
        graph_version: "demo-cambodia-v1.0".to_string(),
        cost_model_version: "tdsp-profiles-v1.0".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::create_demo_graph;

    #[test]
    fn test_cambodia_bounds_validation() {
        assert!(validate_cambodia_bounds(11.5564, 104.9282)); // Phnom Penh
        assert!(!validate_cambodia_bounds(85.0, 0.0)); // North Pole OOB
        assert!(!validate_cambodia_bounds(0.0, 0.0)); // Equator
    }

    #[test]
    fn test_identity_matrix_zero() {
        let graph = create_demo_graph();
        let spatial = SpatialIndex::new(&graph);
        let coord = Coordinate::new(11.5564, 104.9282);

        let res = compute_cost_matrix(
            &graph,
            &spatial,
            &[coord],
            &[coord],
            VehicleProfile::Car,
            CostMetric::Time,
            None,
        )
        .expect("matrix computation should succeed");

        assert_eq!(res.durations.len(), 1);
        assert_eq!(res.durations[0].len(), 1);
        assert_eq!(res.durations[0][0], Some(0.0));
        assert_eq!(res.distances[0][0], Some(0.0));
    }

    #[test]
    fn test_asymmetric_matrix_costs() {
        let graph = create_demo_graph();
        let spatial = SpatialIndex::new(&graph);
        let p1 = Coordinate::new(11.5564, 104.9282); // Independence Monument
        let p2 = Coordinate::new(11.5760, 104.9230); // Wat Phnom

        let res = compute_cost_matrix(
            &graph,
            &spatial,
            &[p1, p2],
            &[p1, p2],
            VehicleProfile::Car,
            CostMetric::Time,
            None,
        )
        .expect("matrix computation should succeed");

        assert_eq!(res.durations.len(), 2);
        assert_eq!(res.durations[0].len(), 2);
        assert_eq!(res.durations[0][0], Some(0.0));
        assert_eq!(res.durations[1][1], Some(0.0));
        assert!(res.durations[0][1].is_some());
        assert!(res.durations[1][0].is_some());
    }

    #[test]
    fn test_truck_vs_motorcycle_profile_expressway() {
        let graph = create_demo_graph();
        let spatial = SpatialIndex::new(&graph);
        let pp = Coordinate::new(11.5650, 104.8960); // Techno Flyover (start of expressway)
        let shv = Coordinate::new(10.6253, 103.5234); // Sihanoukville

        // Truck routing
        let truck_res = compute_cost_matrix(
            &graph,
            &spatial,
            &[pp],
            &[shv],
            VehicleProfile::Truck,
            CostMetric::Time,
            None,
        )
        .expect("truck matrix computation should succeed");

        // Motorcycle routing
        let moto_res = compute_cost_matrix(
            &graph,
            &spatial,
            &[pp],
            &[shv],
            VehicleProfile::Motorcycle,
            CostMetric::Time,
            None,
        )
        .expect("moto matrix computation should succeed");

        assert!(truck_res.durations[0][0].is_some());
        assert!(moto_res.durations[0][0].is_some());
    }
}
