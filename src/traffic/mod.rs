//! Time-dependent traffic congestion modeling.
//!
//! Simulates realistic time-of-day traffic dynamics across urban centers
//! and national highways, providing weights for Time-Dependent Shortest Path
//! (TDSP) calculations.
//!
//! ## Mathematical Model
//!
//! The effective travel duration along edge $e = (u, v)$ at departure time $t$ is:
//!
//! $$D(e, t) = D_0(e) \cdot C(e, t)$$
//!
//! where $D_0(e)$ is the baseline free-flow duration and $C(e, t) \ge 1.0$ is the
//! congestion multiplier. $C(e, t)$ combines:
//! 1. **Temporal factor $T(t)$**: Gaussian-shaped congestion spikes around morning
//!    rush (07:30–09:00) and evening rush (17:00–19:00).
//! 2. **Spatial factor $S(u)$**: Radial density decay centered around urban hubs
//!    (e.g., central Phnom Penh, Siem Reap).
//!
//! Expressways and ring roads with grade separation experience minimal congestion,
//! allowing the routing engine to discover realistic detours during peak hours.

use crate::geo::haversine;
use crate::graph::Coordinate;

/// Known urban congestion centers with (latitude, longitude, radius in meters).
const CONGESTION_HUBS: &[(f64, f64, f64, f64)] = &[
    // (lat, lon, radius_m, max_severity)
    (11.5564, 104.9282, 12_000.0, 3.8), // Phnom Penh Central
    (13.3671, 103.8448, 6_000.0, 2.5),  // Siem Reap Old Market / Central
    (13.0957, 103.2022, 5_000.0, 2.0),  // Battambang Central
    (10.6253, 103.5234, 6_000.0, 2.2),  // Sihanoukville Port/City
];

/// Maximum vehicular speed on the road network in meters per second (120 km/h).
/// Used to maintain an admissible heuristic for A* time-based routing.
pub const MAX_NETWORK_SPEED_MS: f64 = 120.0 / 3.6;

/// Parses a 24-hour time string (e.g. `"08:30"`, `"17:15"`) into minutes since midnight [0..1439].
pub fn parse_time_of_day(time_str: &str) -> Option<u32> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hours: u32 = parts[0].trim().parse().ok()?;
    let minutes: u32 = parts[1].trim().parse().ok()?;
    if hours < 24 && minutes < 60 {
        Some(hours * 60 + minutes)
    } else {
        None
    }
}

/// Evaluates the temporal congestion factor $T(t) \in [1.0, 4.0]$ based on minutes from midnight.
pub fn temporal_factor(minutes_from_midnight: u32) -> f64 {
    let t = minutes_from_midnight as f64;

    // Morning rush: peak at 08:15 (495 min), stddev ~ 45 min
    let morning_diff = (t - 495.0) / 45.0;
    let morning_spike = (-0.5 * morning_diff * morning_diff).exp();

    // Evening rush: peak at 17:45 (1065 min), stddev ~ 60 min
    let evening_diff = (t - 1065.0) / 60.0;
    let evening_spike = (-0.5 * evening_diff * evening_diff).exp();

    // Midday plateau: between 11:30 and 13:30 (lunch rush)
    let lunch_diff = (t - 750.0) / 50.0;
    let lunch_spike = (-0.5 * lunch_diff * lunch_diff).exp();

    1.0 + 2.5 * morning_spike + 3.0 * evening_spike + 0.8 * lunch_spike
}

/// Computes the spatial congestion weight for a coordinate based on proximity to urban hubs.
pub fn spatial_factor(coord: &Coordinate) -> f64 {
    let mut max_urban_factor = 0.0;

    for &(hub_lat, hub_lon, radius, severity) in CONGESTION_HUBS {
        let dist = haversine::distance(coord.lat, coord.lon, hub_lat, hub_lon);
        if dist < radius {
            // Smooth cubic smoothstep decay from center to periphery
            let x = 1.0 - (dist / radius);
            let decay = x * x * (3.0 - 2.0 * x);
            let factor = decay * (severity - 1.0);
            if factor > max_urban_factor {
                max_urban_factor = factor;
            }
        }
    }

    max_urban_factor
}

/// Computes the overall congestion multiplier $C(e, t)$ for an edge departing at time $t$.
pub fn congestion_multiplier(coord: &Coordinate, minutes_from_midnight: Option<u32>) -> f64 {
    let minutes = match minutes_from_midnight {
        Some(m) => m,
        None => return 1.0, // Free-flow baseline
    };

    let t_factor = temporal_factor(minutes);
    let s_factor = spatial_factor(coord);

    // Highway segments far from cities remain close to free-flow even at rush hour
    1.0 + (t_factor - 1.0) * s_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_of_day() {
        assert_eq!(parse_time_of_day("08:30"), Some(510));
        assert_eq!(parse_time_of_day("00:00"), Some(0));
        assert_eq!(parse_time_of_day("23:59"), Some(1439));
        assert_eq!(parse_time_of_day("25:00"), None);
        assert_eq!(parse_time_of_day("invalid"), None);
    }

    #[test]
    fn test_temporal_factor_spikes() {
        let night = temporal_factor(120); // 02:00 AM
        assert!((night - 1.0).abs() < 0.01, "Night should be free-flow");

        let morning_rush = temporal_factor(495); // 08:15 AM
        assert!(morning_rush >= 3.0, "Morning rush should have heavy multiplier");

        let evening_rush = temporal_factor(1065); // 17:45 PM
        assert!(evening_rush >= 3.5, "Evening rush should have severe multiplier");
    }

    #[test]
    fn test_spatial_factor_decay() {
        let pp_center = Coordinate::new(11.5564, 104.9282);
        let s_center = spatial_factor(&pp_center);
        assert!(s_center > 2.0, "Central PP should have high spatial factor");

        let countryside = Coordinate::new(12.5000, 104.2000);
        let s_country = spatial_factor(&countryside);
        assert_eq!(s_country, 0.0, "Rural area should have zero congestion factor");
    }
}
