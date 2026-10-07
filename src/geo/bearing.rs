//! Geographic bearing and turn angle calculations.
//!
//! Provides initial compass bearings, relative turn angles, and turn penalties
//! between sequential road segments, as well as destination point projection.

use std::f64::consts::PI;

/// Mean radius of the Earth in meters (WGS84 approximation).
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculates the initial compass bearing in degrees (0° to 360°)
/// from point 1 to point 2 along a great-circle path.
///
/// # Arguments
///
/// * `lat1`, `lon1` - Latitude and longitude of the starting point (degrees).
/// * `lat2`, `lon2` - Latitude and longitude of the destination point (degrees).
///
/// # Returns
///
/// Compass bearing in degrees, where 0° is North, 90° is East, 180° is South, and 270° is West.
pub fn initial_bearing(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1 * PI / 180.0;
    let lat2_rad = lat2 * PI / 180.0;
    let dlon_rad = (lon2 - lon1) * PI / 180.0;

    let y = dlon_rad.sin() * lat2_rad.cos();
    let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * dlon_rad.cos();

    let bearing_rad = y.atan2(x);
    let bearing_deg = bearing_rad * 180.0 / PI;

    (bearing_deg + 360.0) % 360.0
}

/// Calculates the signed turn angle between two consecutive road segments.
///
/// # Arguments
///
/// * `bearing_in` - Bearing of the incoming road segment (0° to 360°).
/// * `bearing_out` - Bearing of the outgoing road segment (0° to 360°).
///
/// # Returns
///
/// Signed angle in degrees in the range `[-180.0, 180.0]`:
/// - Near 0°: straight ahead
/// - Positive: right turn (e.g., +90° = sharp right)
/// - Negative: left turn (e.g., -90° = sharp left)
/// - Near ±180°: U-turn
pub fn turn_angle(bearing_in: f64, bearing_out: f64) -> f64 {
    (bearing_out - bearing_in + 540.0) % 360.0 - 180.0
}

/// Estimates the turn penalty in seconds based on turn angle.
///
/// Incorporates traffic geometry: right turns are faster than left turns
/// in right-hand driving traffic countries (like Cambodia, US, Europe).
///
/// # Arguments
///
/// * `turn_angle_deg` - Signed turn angle in degrees from [`turn_angle`].
///
/// # Returns
///
/// Delay in seconds to add to the travel duration.
pub fn turn_penalty_seconds(turn_angle_deg: f64) -> f64 {
    let abs_angle = turn_angle_deg.abs();
    if abs_angle < 35.0 {
        // Straight or gentle curve
        0.0
    } else if abs_angle >= 135.0 {
        // Sharp hairpin or U-turn
        15.0
    } else if abs_angle < 70.0 {
        // Slight turn
        2.5
    } else if turn_angle_deg > 0.0 {
        // Right turn (with traffic flow)
        4.0
    } else {
        // Left turn (crossing oncoming traffic)
        8.0
    }
}

/// Computes the destination coordinates given a starting point, initial bearing,
/// and distance in meters.
///
/// # Arguments
///
/// * `lat`, `lon` - Starting coordinates in degrees.
/// * `bearing_deg` - Compass bearing in degrees (0° = North).
/// * `distance_m` - Great circle distance in meters.
///
/// # Returns
///
/// `(dest_lat, dest_lon)` in degrees.
pub fn destination_point(lat: f64, lon: f64, bearing_deg: f64, distance_m: f64) -> (f64, f64) {
    let lat_rad = lat * PI / 180.0;
    let lon_rad = lon * PI / 180.0;
    let bearing_rad = bearing_deg * PI / 180.0;
    let angular_dist = distance_m / EARTH_RADIUS_M;

    let dest_lat_rad = (lat_rad.sin() * angular_dist.cos()
        + lat_rad.cos() * angular_dist.sin() * bearing_rad.cos())
    .asin();

    let dest_lon_rad = lon_rad
        + (bearing_rad.sin() * angular_dist.sin() * lat_rad.cos())
            .atan2(angular_dist.cos() - lat_rad.sin() * dest_lat_rad.sin());

    let dest_lat = dest_lat_rad * 180.0 / PI;
    let dest_lon = (dest_lon_rad * 180.0 / PI + 540.0) % 360.0 - 180.0;

    (dest_lat, dest_lon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_bearing_cardinal() {
        // North
        let b_north = initial_bearing(0.0, 0.0, 1.0, 0.0);
        assert!((b_north - 0.0).abs() < 1e-4 || (b_north - 360.0).abs() < 1e-4);

        // East
        let b_east = initial_bearing(0.0, 0.0, 0.0, 1.0);
        assert!((b_east - 90.0).abs() < 1e-4);

        // South
        let b_south = initial_bearing(1.0, 0.0, 0.0, 0.0);
        assert!((b_south - 180.0).abs() < 1e-4);

        // West
        let b_west = initial_bearing(0.0, 1.0, 0.0, 0.0);
        assert!((b_west - 270.0).abs() < 1e-4);
    }

    #[test]
    fn test_turn_angles() {
        // Straight
        assert_eq!(turn_angle(90.0, 90.0), 0.0);

        // 90 deg right turn (heading North then turning East)
        assert!((turn_angle(0.0, 90.0) - 90.0).abs() < 1e-4);

        // 90 deg left turn (heading North then turning West)
        assert!((turn_angle(0.0, 270.0) - (-90.0)).abs() < 1e-4);

        // U-turn
        let u_turn = turn_angle(0.0, 180.0).abs();
        assert!((u_turn - 180.0).abs() < 1e-4);
    }

    #[test]
    fn test_turn_penalties() {
        assert_eq!(turn_penalty_seconds(10.0), 0.0);
        assert_eq!(turn_penalty_seconds(90.0), 4.0); // Right
        assert_eq!(turn_penalty_seconds(-90.0), 8.0); // Left
        assert_eq!(turn_penalty_seconds(170.0), 15.0); // U-turn
    }

    #[test]
    fn test_destination_point() {
        let (lat1, lon1) = (11.5564, 104.9282);
        let dist = 10_000.0; // 10 km North
        let (lat2, lon2) = destination_point(lat1, lon1, 0.0, dist);

        assert!(lat2 > lat1);
        assert!((lon2 - lon1).abs() < 0.001);

        let d_actual = crate::geo::haversine::distance(lat1, lon1, lat2, lon2);
        assert!((d_actual - dist).abs() < 1.0);
    }
}
