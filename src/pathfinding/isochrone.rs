//! Isochrone generation and travel-time reachability polygons.
//!
//! An isochrone map depicts the area accessible from a point within one or more
//! specified time thresholds. This module performs bounded Dijkstra exploration
//! and derives smooth boundary polygons (GeoJSON MultiPolygons/Polygons) for each
//! travel time interval.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::geo::bearing::{destination_point, initial_bearing};
use crate::geo::haversine;
use crate::graph::RoadGraph;
use crate::traffic;

use super::types::RoutingOptions;

/// Represents an isochrone polygon boundary for a specific time threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsochroneBucket {
    /// Time threshold in minutes (e.g., 10, 20, 30).
    pub time_minutes: u32,
    /// Number of road network nodes reached within this threshold.
    pub nodes_reached: usize,
    /// Estimated surface area enclosed by the polygon in square kilometers.
    pub area_sq_km: f64,
    /// Recommended fill/stroke hex color for visualization.
    pub color: String,
    /// Polygon boundary coordinates in `[longitude, latitude]` format (GeoJSON standard).
    /// First and last coordinates are identical to close the linear ring.
    pub coordinates: Vec<[f64; 2]>,
}

/// The complete result of an isochrone computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsochroneResult {
    /// Origin coordinate `[longitude, latitude]`.
    pub center: [f64; 2],
    /// Snapped starting node ID.
    pub start_node: u32,
    /// Isochrone contour buckets ordered by ascending time.
    pub buckets: Vec<IsochroneBucket>,
    /// Total road nodes visited during exploration.
    pub total_nodes_visited: usize,
    /// Computation duration.
    pub query_time: Duration,
}

impl IsochroneResult {
    /// Converts the isochrone result into a standard GeoJSON `FeatureCollection`.
    pub fn to_geojson(&self) -> serde_json::Value {
        let features: Vec<serde_json::Value> = self
            .buckets
            .iter()
            .map(|bucket| {
                serde_json::json!({
                    "type": "Feature",
                    "properties": {
                        "time_minutes": bucket.time_minutes,
                        "nodes_reached": bucket.nodes_reached,
                        "area_sq_km": (bucket.area_sq_km * 10.0).round() / 10.0,
                        "color": bucket.color,
                        "fillOpacity": 0.25,
                        "strokeWeight": 2
                    },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [bucket.coordinates]
                    }
                })
            })
            .collect();

        serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
            "properties": {
                "center": self.center,
                "total_nodes_visited": self.total_nodes_visited,
                "query_time_ms": self.query_time.as_secs_f64() * 1000.0
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct QueueEntry {
    node: u32,
    time_s: f64,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.time_s == other.time_s
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
        // Min-heap
        other
            .time_s
            .partial_cmp(&self.time_s)
            .unwrap_or(Ordering::Equal)
    }
}

/// Generates a palette color based on time threshold.
fn bucket_color(minutes: u32) -> String {
    match minutes {
        0..=10 => "#10b981".to_string(),  // Emerald Green
        11..=20 => "#06b6d4".to_string(), // Cyan
        21..=30 => "#3b82f6".to_string(), // Blue
        31..=45 => "#f59e0b".to_string(), // Amber
        46..=60 => "#f97316".to_string(), // Orange
        _ => "#ef4444".to_string(),       // Crimson
    }
}

/// Computes travel-time isochrones from a starting node.
///
/// # Arguments
///
/// * `graph` - The road network graph.
/// * `start_node` - Internal node ID to explore from.
/// * `time_buckets_minutes` - Slice of thresholds in minutes (e.g. `&[10, 20, 30]`).
/// * `options` - Routing options for traffic congestion departure times.
///
/// # Returns
///
/// [`IsochroneResult`] containing layered polygons and metrics.
pub fn compute_isochrones(
    graph: &RoadGraph,
    start_node: u32,
    time_buckets_minutes: &[u32],
    options: &RoutingOptions,
) -> Option<IsochroneResult> {
    let start_time = Instant::now();
    let num_nodes = graph.node_count();

    if (start_node as usize) >= num_nodes || time_buckets_minutes.is_empty() {
        return None;
    }

    let start_coord = graph.get_coord(start_node)?;
    let center_lat = start_coord.lat;
    let center_lon = start_coord.lon;

    let mut sorted_buckets = time_buckets_minutes.to_vec();
    sorted_buckets.sort_unstable();
    sorted_buckets.dedup();

    let max_time_s = (*sorted_buckets.last().unwrap_or(&30) as f64) * 60.0;

    let mut dist_s = vec![f64::INFINITY; num_nodes];
    dist_s[start_node as usize] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(QueueEntry {
        node: start_node,
        time_s: 0.0,
    });

    let mut nodes_visited = 0;

    while let Some(QueueEntry { node, time_s }) = heap.pop() {
        if time_s > dist_s[node as usize] {
            continue;
        }

        nodes_visited += 1;

        if time_s > max_time_s {
            // Priority queue yields monotonically increasing costs; once min > max_time_s, stop
            break;
        }

        for edge in graph.neighbors(node) {
            let target = edge.target;
            let target_coord = graph.get_coord(target);

            // Dynamic Time-Dependent Shortest Path (TDSP):
            // Advance departure time by accumulated elapsed travel time
            let edge_departure_minutes = options
                .departure_minutes
                .map(|dep| (dep + (time_s / 60.0).floor() as u32) % 1440);

            let edge_duration = match target_coord {
                Some(coord) => {
                    let multiplier = traffic::congestion_multiplier(coord, edge_departure_minutes);
                    edge.duration_s * multiplier
                }
                None => edge.duration_s,
            };

            let new_time = time_s + edge_duration;
            if new_time < dist_s[target as usize] {
                dist_s[target as usize] = new_time;
                heap.push(QueueEntry {
                    node: target,
                    time_s: new_time,
                });
            }
        }
    }

    // Build polygons for each bucket
    let num_sectors = 36; // 10 degrees per sector
    let sector_angle_step = 360.0 / (num_sectors as f64);

    let mut buckets = Vec::new();

    for &minutes in &sorted_buckets {
        let threshold_s = (minutes as f64) * 60.0;

        // Collect reachable coordinates
        let mut sector_max_dist = vec![0.0f64; num_sectors];
        let mut nodes_in_bucket = 0;

        for (id, &time) in dist_s.iter().enumerate() {
            if time <= threshold_s {
                nodes_in_bucket += 1;
                if let Some(coord) = graph.get_coord(id as u32) {
                    if (coord.lat - center_lat).abs() < 1e-7
                        && (coord.lon - center_lon).abs() < 1e-7
                    {
                        continue;
                    }

                    let d = haversine::distance(center_lat, center_lon, coord.lat, coord.lon);
                    let bearing = initial_bearing(center_lat, center_lon, coord.lat, coord.lon);
                    let sector_idx = ((bearing / sector_angle_step).floor() as usize) % num_sectors;

                    if d > sector_max_dist[sector_idx] {
                        sector_max_dist[sector_idx] = d;
                    }
                }
            }
        }

        // Fill empty sectors with nearby sector interpolation
        let default_min_radius = (minutes as f64) * 150.0; // fallback base radius
        let mut filled_dist = sector_max_dist.clone();

        for (i, dist) in filled_dist.iter_mut().enumerate() {
            if *dist < 10.0 {
                // Find non-zero left and right
                let mut left_val = default_min_radius;
                for step in 1..num_sectors {
                    let idx = (i + num_sectors - step) % num_sectors;
                    if sector_max_dist[idx] > 10.0 {
                        left_val = sector_max_dist[idx];
                        break;
                    }
                }

                let mut right_val = default_min_radius;
                for step in 1..num_sectors {
                    let idx = (i + step) % num_sectors;
                    if sector_max_dist[idx] > 10.0 {
                        right_val = sector_max_dist[idx];
                        break;
                    }
                }

                *dist = (left_val + right_val) / 2.0;
            }
        }

        // Apply 3-point circular smoothing filter for aesthetic, natural contours
        let mut smoothed_dist = vec![0.0f64; num_sectors];
        for i in 0..num_sectors {
            let prev = filled_dist[(i + num_sectors - 1) % num_sectors];
            let curr = filled_dist[i];
            let next = filled_dist[(i + 1) % num_sectors];
            smoothed_dist[i] = 0.25 * prev + 0.50 * curr + 0.25 * next;
        }

        // Project boundary coordinates
        let mut polygon_coords = Vec::with_capacity(num_sectors + 1);
        let mut projected_xy = Vec::with_capacity(num_sectors);

        for (i, &radius) in smoothed_dist.iter().enumerate() {
            let bearing = (i as f64 + 0.5) * sector_angle_step;
            let (lat, lon) = destination_point(center_lat, center_lon, bearing, radius);
            // GeoJSON coordinates are [lon, lat]
            polygon_coords.push([lon, lat]);

            // Planar approximation for area calculation in meters
            let bearing_rad = bearing.to_radians();
            let x = radius * bearing_rad.sin();
            let y = radius * bearing_rad.cos();
            projected_xy.push((x, y));
        }

        // Close polygon ring (first point = last point)
        if let Some(&first) = polygon_coords.first() {
            polygon_coords.push(first);
        }

        // Calculate enclosed polygon area via Shoelace formula
        let mut area_sq_m = 0.0;
        let n = projected_xy.len();
        for i in 0..n {
            let j = (i + 1) % n;
            area_sq_m += projected_xy[i].0 * projected_xy[j].1;
            area_sq_m -= projected_xy[j].0 * projected_xy[i].1;
        }
        let area_sq_km = (area_sq_m.abs() / 2.0) / 1_000_000.0;

        buckets.push(IsochroneBucket {
            time_minutes: minutes,
            nodes_reached: nodes_in_bucket,
            area_sq_km,
            color: bucket_color(minutes),
            coordinates: polygon_coords,
        });
    }

    Some(IsochroneResult {
        center: [center_lon, center_lat],
        start_node,
        buckets,
        total_nodes_visited: nodes_visited,
        query_time: start_time.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_isochrone_computation() {
        let mut builder = GraphBuilder::new();
        // Central hub
        builder.add_node(1, 11.5564, 104.9282);
        // North node ~1.1 km away (60 seconds at 66 km/h)
        builder.add_node(2, 11.5664, 104.9282);
        // East node
        builder.add_node(3, 11.5564, 104.9382);
        // Further node ~10 km away
        builder.add_node(4, 11.6464, 104.9282);

        builder.add_way(&[1, 2], false, 60.0);
        builder.add_way(&[1, 3], false, 60.0);
        builder.add_way(&[2, 4], false, 600.0);

        let graph = builder.build();

        let res = compute_isochrones(&graph, 0, &[5, 15], &RoutingOptions::default())
            .expect("Isochrone should succeed");

        assert_eq!(res.buckets.len(), 2);
        assert_eq!(res.buckets[0].time_minutes, 5);
        assert_eq!(res.buckets[1].time_minutes, 15);

        // All polygons should be closed rings (first coord == last coord)
        for bucket in &res.buckets {
            assert!(bucket.coordinates.len() > 3);
            assert_eq!(bucket.coordinates.first(), bucket.coordinates.last());
            assert!(bucket.area_sq_km > 0.0);
        }

        // GeoJSON output should be valid
        let geojson = res.to_geojson();
        assert_eq!(geojson["type"], "FeatureCollection");
        assert_eq!(geojson["features"].as_array().unwrap().len(), 2);
    }
}
