//! Fleet vehicle profile definitions and road network constraints.
//!
//! Provides vehicle profiles (Car, Van, Truck, Motorcycle) that govern:
//! - Accessible road classes (e.g. Motorcycle prohibited from Motorway, Truck prohibited from Residential)
//! - Speed caps per road class (e.g. Truck capped at 80 km/h on Motorway, 60 km/h on Primary)
//! - Turn penalties and U-turn constraints (e.g. Truck sharp turn penalty = 60.0s, Motorcycle = 3.0s)

use crate::graph::RoadClass;

/// Vehicle profiles for routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VehicleProfile {
    #[default]
    Car,
    Van,
    Truck,
    Motorcycle,
}

impl VehicleProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            VehicleProfile::Car => "car",
            VehicleProfile::Van => "van",
            VehicleProfile::Truck => "truck",
            VehicleProfile::Motorcycle => "motorcycle",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "car" => Some(VehicleProfile::Car),
            "van" => Some(VehicleProfile::Van),
            "truck" => Some(VehicleProfile::Truck),
            "motorcycle" => Some(VehicleProfile::Motorcycle),
            _ => None,
        }
    }

    /// Determines if this vehicle profile is legally permitted on the specified road class.
    pub fn is_road_allowed(&self, road_class: RoadClass) -> bool {
        match self {
            VehicleProfile::Car => !matches!(road_class, RoadClass::Other),
            VehicleProfile::Van => !matches!(road_class, RoadClass::LivingStreet | RoadClass::Other),
            VehicleProfile::Truck => matches!(
                road_class,
                RoadClass::Motorway | RoadClass::Trunk | RoadClass::Primary | RoadClass::Secondary | RoadClass::Tertiary
            ),
            VehicleProfile::Motorcycle => !matches!(road_class, RoadClass::Motorway | RoadClass::Other),
        }
    }

    /// Returns the legal maximum speed cap (km/h) for this vehicle profile on the given road class.
    pub fn speed_cap_kmh(&self, road_class: RoadClass) -> f64 {
        match self {
            VehicleProfile::Car => 120.0,
            VehicleProfile::Van => 90.0,
            VehicleProfile::Truck => match road_class {
                RoadClass::Motorway => 80.0,
                RoadClass::Trunk => 70.0,
                RoadClass::Primary => 60.0,
                RoadClass::Secondary => 50.0,
                RoadClass::Tertiary => 40.0,
                _ => 40.0,
            },
            VehicleProfile::Motorcycle => 60.0,
        }
    }

    /// Calculates the effective speed (km/h) clamped to the vehicle's speed cap.
    pub fn effective_speed_kmh(&self, road_class: RoadClass, base_speed_kmh: f64) -> f64 {
        let cap = self.speed_cap_kmh(road_class);
        base_speed_kmh.min(cap).max(5.0)
    }

    /// Estimates turn penalty in seconds based on signed turn angle.
    pub fn turn_penalty_seconds(&self, turn_angle_deg: f64) -> f64 {
        let abs_angle = turn_angle_deg.abs();
        if abs_angle < 35.0 {
            0.0
        } else {
            match self {
                VehicleProfile::Motorcycle => {
                    if abs_angle >= 135.0 {
                        3.0
                    } else {
                        1.5
                    }
                }
                VehicleProfile::Truck => {
                    if abs_angle >= 135.0 {
                        60.0
                    } else {
                        15.0
                    }
                }
                VehicleProfile::Car | VehicleProfile::Van => {
                    if abs_angle >= 135.0 {
                        15.0
                    } else {
                        4.0
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_profile_restrictions() {
        let car = VehicleProfile::Car;
        let truck = VehicleProfile::Truck;
        let moto = VehicleProfile::Motorcycle;

        assert!(car.is_road_allowed(RoadClass::Motorway));
        assert!(car.is_road_allowed(RoadClass::Residential));

        assert!(!moto.is_road_allowed(RoadClass::Motorway));
        assert!(moto.is_road_allowed(RoadClass::Residential));

        assert!(truck.is_road_allowed(RoadClass::Motorway));
        assert!(!truck.is_road_allowed(RoadClass::Residential));
        assert!(!truck.is_road_allowed(RoadClass::LivingStreet));
    }

    #[test]
    fn test_vehicle_profile_speed_caps() {
        let truck = VehicleProfile::Truck;
        assert_eq!(truck.speed_cap_kmh(RoadClass::Motorway), 80.0);
        assert_eq!(truck.speed_cap_kmh(RoadClass::Primary), 60.0);
        assert_eq!(truck.effective_speed_kmh(RoadClass::Motorway, 120.0), 80.0);
    }

    #[test]
    fn test_vehicle_profile_turn_penalties() {
        let moto = VehicleProfile::Motorcycle;
        let truck = VehicleProfile::Truck;

        assert_eq!(moto.turn_penalty_seconds(150.0), 3.0);
        assert_eq!(truck.turn_penalty_seconds(150.0), 60.0);
        assert!(truck.turn_penalty_seconds(150.0) > moto.turn_penalty_seconds(150.0) * 10.0);
    }
}
