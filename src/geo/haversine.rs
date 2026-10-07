//! Haversine distance calculation for geographic coordinates.
//!
//! The Haversine formula determines the great-circle distance between two
//! points on a sphere given their latitudes and longitudes. This provides
//! an accurate distance measure for routing and serves as the admissible
//! heuristic for the A* algorithm.

use std::f64::consts::PI;

/// Mean radius of the Earth in meters (WGS84 approximation).
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculates the great-circle distance in meters between two geographic
/// coordinates using the Haversine formula.
///
/// # Arguments
///
/// * `lat1`, `lon1` - Latitude and longitude of the first point (degrees).
/// * `lat2`, `lon2` - Latitude and longitude of the second point (degrees).
///
/// # Returns
///
/// Distance in meters.
///
/// # Mathematical Basis
///
/// The Haversine formula:
///
/// ```text
/// a = sin²(Δlat/2) + cos(lat1) · cos(lat2) · sin²(Δlon/2)
/// c = 2 · atan2(√a, √(1−a))
/// d = R · c
/// ```
///
/// This heuristic is admissible for A* because the straight-line (great-circle)
/// distance is always ≤ the actual road distance, satisfying h(n) ≤ d(n).
pub fn distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let dlat = (lat2 - lat1) * PI / 180.0;
    let dlon = (lon2 - lon1) * PI / 180.0;

    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_M * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_distance() {
        let d = distance(11.5564, 104.9282, 11.5564, 104.9282);
        assert!(d.abs() < 1.0, "Same point should have ~0 distance");
    }

    #[test]
    fn test_phnom_penh_to_siem_reap() {
        // Phnom Penh (11.5564, 104.9282) to Siem Reap (13.3671, 103.8448)
        // Expected: ~260–315 km straight-line
        let d = distance(11.5564, 104.9282, 13.3671, 103.8448);
        let d_km = d / 1000.0;
        assert!(
            (220.0..=350.0).contains(&d_km),
            "Phnom Penh to Siem Reap should be ~260-315 km, got {d_km:.1} km"
        );
    }

    #[test]
    fn test_symmetry() {
        let d1 = distance(11.5564, 104.9282, 13.3671, 103.8448);
        let d2 = distance(13.3671, 103.8448, 11.5564, 104.9282);
        assert!((d1 - d2).abs() < 0.01, "Haversine should be symmetric");
    }

    #[test]
    fn test_short_distance() {
        // Two points ~500m apart in Phnom Penh
        let d = distance(11.5564, 104.9282, 11.5600, 104.9282);
        assert!(
            (300.0..=500.0).contains(&d),
            "Short distance should be ~400m, got {d:.1}m"
        );
    }
}
