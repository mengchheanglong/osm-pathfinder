//! Demo graph generator for out-of-the-box exploration without a large PBF file.
//!
//! Generates a realistic road network connecting major Cambodian cities,
//! national highway corridors (NR1, NR3, NR4, NR5, NR6, NR7, Expressway),
//! and urban street grids in Phnom Penh and Siem Reap.

use super::builder::GraphBuilder;
use super::types::{RoadClass, RoadGraph};

/// Creates a demo road network spanning Cambodia with national highway corridors.
pub fn create_demo_graph() -> RoadGraph {
    let mut builder = GraphBuilder::new();

    // -----------------------------------------------------------------------
    // Phnom Penh Urban Delivery Network & Arterials
    // -----------------------------------------------------------------------
    // Core Landmarks
    builder.add_node(1001, 11.5564, 104.9282); // Independence Monument (Norodom x Sihanouk)
    builder.add_node(1002, 11.5760, 104.9230); // Wat Phnom / Chroy Changvar Roundabout
    builder.add_node(1003, 11.5450, 104.9310); // Chamkarmon (Norodom x Mao Tse Toung)
    builder.add_node(1004, 11.5650, 104.8960); // Techno Flyover (Russian Blvd x St 271)

    // Delivery Hubs & Depots
    builder.add_node(1010, 11.5680, 104.9223); // Central Market (Depot A)
    builder.add_node(1020, 11.5435, 104.9142); // Russian Market / Toul Tompoung (Depot B)
    builder.add_node(1030, 11.5621, 104.9160); // Bak Touk / E-Commerce Central Hub

    // Monivong Boulevard (North-South Main Commercial Spine)
    builder.add_node(1101, 11.5685, 104.9225); // Monivong x Kampuchea Krom (Central Market East)
    builder.add_node(1102, 11.5625, 104.9226); // Monivong x Charles de Gaulle
    builder.add_node(1103, 11.5564, 104.9225); // Monivong x Sihanouk Blvd
    builder.add_node(1104, 11.5490, 104.9230); // Monivong x St 288 (BKK1 West)
    builder.add_node(1105, 11.5410, 104.9240); // Bokor Junction (Monivong x Mao Tse Toung)
    builder.add_node(1106, 11.5360, 104.9270); // Monivong South (Boeung Keng Kang South)
    builder.add_node(1107, 11.5310, 104.9310); // Kbal Tnal Interchange / Monivong Bridge
    builder.add_way(&[1002, 1101, 1102, 1103, 1104, 1105, 1106, 1107], false, 40.0);

    // Norodom Boulevard (North-South Grand Avenue)
    builder.add_node(1110, 11.5760, 104.9265); // Wat Phnom East / Norodom North
    builder.add_node(1111, 11.5670, 104.9270); // Norodom x St 130 (Central Daun Penh)
    builder.add_node(1112, 11.5615, 104.9275); // Norodom x St 178 (National Museum)
    builder.add_node(1113, 11.5500, 104.9295); // Norodom x St 294 (BKK1 East)
    builder.add_node(1114, 11.5380, 104.9325); // Norodom South (Tonle Bassac Embassy Zone)
    builder.add_way(&[1110, 1111, 1112, 1001, 1113, 1003, 1114, 1107], false, 45.0);
    builder.add_way(&[1002, 1110], false, 35.0); // Wat Phnom circle

    // Sisowath Quay & Riverside Waterfront
    builder.add_node(1120, 11.5750, 104.9315); // Night Market / Old Market Quay
    builder.add_node(1121, 11.5695, 104.9312); // Riverside / Phsar Kandal
    builder.add_node(1122, 11.5630, 104.9335); // Royal Palace Quay
    builder.add_node(1123, 11.5520, 104.9380); // NagaWorld / Koh Pich (Diamond Island)
    builder.add_way(&[1120, 1121, 1122, 1123], false, 35.0);
    builder.add_way(&[1110, 1120], false, 35.0);
    builder.add_way(&[1111, 1121], false, 35.0);
    builder.add_way(&[1001, 1123], false, 35.0);

    // Chroy Changvar Bridge
    builder.add_node(1125, 11.5830, 104.9300); // Chroy Changvar Peninsula
    builder.add_way(&[1002, 1125], false, 50.0);

    // Sihanouk Boulevard (East-West Arterial)
    builder.add_node(1130, 11.5564, 104.9250); // Sihanouk x Pasteur (St 51)
    builder.add_node(1131, 11.5564, 104.9150); // Sihanouk x Olympic Stadium East
    builder.add_node(1132, 11.5564, 104.9060); // Olympic Stadium West / St 182
    builder.add_way(&[1123, 1001, 1130, 1103, 1131, 1132], false, 40.0);

    // Kampuchea Krom Boulevard (St 128) & Central Market Access
    builder.add_node(1140, 11.5680, 104.9180); // Central Market West / Depo Market approach
    builder.add_node(1141, 11.5665, 104.9060); // Kampuchea Krom x Nehru Blvd (Depo Market)
    builder.add_way(&[1111, 1101, 1010, 1140, 1141, 1004], false, 40.0);

    // Russian Federation Boulevard (Airport / Sen Sok Express)
    builder.add_node(1150, 11.5740, 104.9130); // French Embassy / Tuol Kork South
    builder.add_node(1151, 11.5630, 104.8870); // RUPP / Royal University
    builder.add_node(1152, 11.5600, 104.8720); // Phsar Dei Huy Flyover
    builder.add_node(1153, 11.5500, 104.8450); // Phnom Penh International Airport (PNH)
    builder.add_way(&[1002, 1150, 1004, 1151, 1152, 1153], false, 60.0);

    // Mao Tse Toung Boulevard
    builder.add_node(1160, 11.5445, 104.9165); // Mao Tse Toung x St 163 (Russian Market north)
    builder.add_node(1161, 11.5475, 104.9060); // Mao Tse Toung x Chinese Embassy
    builder.add_node(1162, 11.5540, 104.8990); // Mao Tse Toung x St 217
    builder.add_way(&[1003, 1105, 1160, 1161, 1162, 1004], false, 45.0);

    // Street 271 (Southern Ring Road / Meanchey Artery)
    builder.add_node(1170, 11.5305, 104.9180); // St 271 x Boeung Tumpun East
    builder.add_node(1171, 11.5305, 104.9085); // St 271 x St 371 (Boeung Tumpun South)
    builder.add_node(1172, 11.5340, 104.8920); // St 271 x St 217 (Steung Meanchey Flyover)
    builder.add_node(1173, 11.5500, 104.8930); // St 271 x Olympic West / St 182
    builder.add_way(&[1107, 1170, 1171, 1172, 1173, 1004], false, 45.0);
    builder.add_way(&[1132, 1173], false, 40.0); // Olympic West connector

    // Russian Market (Toul Tompoung) District & St 163 Corridor
    builder.add_node(1180, 11.5390, 104.9150); // St 163 x St 432 (Russian Market South)
    builder.add_way(&[1160, 1020, 1180, 1170], false, 35.0); // St 163 connecting to St 271
    builder.add_way_with_class(&[1180, 1106], false, 30.0, RoadClass::Residential); // St 432 east to Monivong

    // BKK1 Residential & Commercial Delivery Grid
    builder.add_node(1190, 11.5528, 104.9282); // Pasteur (St 51) x St 288 (BKK1 Central)
    builder.add_node(1191, 11.5510, 104.9245); // St 63 (Trasak Paem) x St 288
    builder.add_way_with_class(&[1130, 1190, 1113], false, 30.0, RoadClass::Residential); // Pasteur (St 51)
    builder.add_way_with_class(&[1104, 1191, 1190, 1113], false, 30.0, RoadClass::Residential); // St 288 east-west
    builder.add_way_with_class(&[1103, 1191, 1105], false, 30.0, RoadClass::Residential); // St 63 north-south

    // Olympic / Bak Touk Central Hub Grid
    builder.add_way(&[1140, 1030, 1131], false, 35.0); // Charles de Gaulle through Central Hub
    builder.add_way(&[1102, 1030, 1141], false, 35.0); // Bak Touk cross connector

    // Tuol Kork & Sen Sok Delivery Zones
    builder.add_node(1200, 11.5732, 104.8984); // Tuol Kork / St 289 (TK Avenue)
    builder.add_node(1201, 11.5850, 104.8900); // Tuol Kork North / St 598
    builder.add_node(1202, 11.5850, 104.8820); // Sen Sok / AEON Mall 2 / St 1003
    builder.add_way(&[1004, 1200, 1201], false, 40.0); // St 289
    builder.add_way(&[1201, 1202], false, 45.0); // St 598 to Sen Sok
    builder.add_way(&[1152, 1202], false, 45.0); // Phsar Dei Huy Flyover to Sen Sok
    builder.add_way(&[1150, 1200], false, 35.0); // French Embassy to TK Avenue

    // -----------------------------------------------------------------------
    // National Road 6 (PP -> Kampong Thom -> Siem Reap)
    // -----------------------------------------------------------------------
    let nr6_nodes = [
        (2001, 11.6500, 104.9100), // Prek Pnov junction
        (2002, 11.8600, 104.8500), // Batheay
        (2003, 12.0800, 104.8800), // Skun junction
        (2004, 12.3500, 104.8900), // Baray
        (2005, 12.7111, 104.8887), // Kampong Thom City
        (2006, 12.9500, 104.5500), // Stoung
        (2007, 13.1200, 104.2500), // Chi Kraeng
        (2008, 13.2500, 104.0500), // Prasat Bakong
        (2009, 13.3671, 103.8448), // Siem Reap Central
    ];

    let mut nr6_ids = vec![1002]; // Start from PP North
    for (id, lat, lon) in nr6_nodes {
        builder.add_node(id, lat, lon);
        nr6_ids.push(id);
    }
    builder.add_way(&nr6_ids, false, 80.0);

    // Siem Reap Urban Grid
    builder.add_node(2010, 13.4125, 103.8670); // Angkor Wat
    builder.add_node(2011, 13.3550, 103.8550); // Pub Street / Old Market
    builder.add_node(2012, 13.3800, 103.8100); // SR Airport area
    builder.add_way(&[2009, 2010], false, 50.0);
    builder.add_way(&[2009, 2011], false, 35.0);
    builder.add_way(&[2009, 2012], false, 50.0);
    builder.add_way(&[2010, 2012], false, 50.0);

    // -----------------------------------------------------------------------
    // National Road 5 (PP -> Kampong Chhnang -> Pursat -> Battambang -> Sisophon)
    // -----------------------------------------------------------------------
    let nr5_nodes = [
        (3001, 11.7200, 104.7500), // Oudong
        (3002, 12.2500, 104.6667), // Kampong Chhnang
        (3003, 12.5333, 103.9167), // Pursat City
        (3004, 12.8500, 103.5500), // Moung Ruessei
        (3005, 13.0957, 103.2022), // Battambang City
        (3006, 13.3500, 103.0800), // Thma Koul
        (3007, 13.5859, 102.9737), // Sisophon (Banteay Meanchey)
        (3008, 13.6558, 102.5647), // Poipet (Thai border)
    ];

    let mut nr5_ids = vec![1004]; // Start from PP West
    for (id, lat, lon) in nr5_nodes {
        builder.add_node(id, lat, lon);
        nr5_ids.push(id);
    }
    builder.add_way(&nr5_ids, false, 80.0);

    // Connect Sisophon to Siem Reap via NR6 West
    builder.add_node(3010, 13.5000, 103.4500); // Kralanh
    builder.add_way(&[3007, 3010, 2009], false, 80.0);

    // -----------------------------------------------------------------------
    // National Road 4 & Expressway (PP -> Sihanoukville)
    // -----------------------------------------------------------------------
    // NR4 National Highway
    let nr4_nodes = [
        (4001, 11.4500, 104.6500), // Kampong Speu
        (4002, 11.2000, 104.1000), // Pich Nil pass
        (4003, 10.9500, 103.8500), // Prey Nob
        (4004, 10.6253, 103.5234), // Sihanoukville Central
    ];

    let mut nr4_ids = vec![1003]; // Start PP South
    for (id, lat, lon) in nr4_nodes {
        builder.add_node(id, lat, lon);
        nr4_ids.push(id);
    }
    builder.add_way(&nr4_ids, false, 80.0);

    // PP-Sihanoukville Expressway (Express highway: 120 km/h)
    let exp_nodes = [
        (4101, 11.4800, 104.6800),
        (4102, 11.1500, 104.0500),
        (4103, 10.8500, 103.7500),
    ];
    for (id, lat, lon) in exp_nodes {
        builder.add_node(id, lat, lon);
    }
    builder.add_way_with_class(&[1004, 4101, 4102, 4103, 4004], false, 120.0, RoadClass::Motorway);

    // -----------------------------------------------------------------------
    // National Road 3 (PP -> Kampot -> Kep)
    // -----------------------------------------------------------------------
    let nr3_nodes = [
        (5001, 11.3500, 104.7500), // Takeo junction
        (5002, 11.0000, 104.5500), // Chhouk
        (5003, 10.6104, 104.1815), // Kampot City
        (5004, 10.4829, 104.2944), // Kep Coastal City
    ];

    let mut nr3_ids = vec![1003];
    for (id, lat, lon) in nr3_nodes {
        builder.add_node(id, lat, lon);
        nr3_ids.push(id);
    }
    builder.add_way(&nr3_ids, false, 75.0);

    // Connect Kampot to Sihanoukville via coastal NR48/NR3 corridor
    builder.add_way(&[5003, 4003], false, 70.0);

    // -----------------------------------------------------------------------
    // National Road 7 & Eastern Cambodia (Skun -> Kampong Cham -> Kratie)
    // -----------------------------------------------------------------------
    let nr7_nodes = [
        (6001, 11.9924, 105.4645), // Kampong Cham City (Mekong River)
        (6002, 12.2000, 105.7500), // Chhlong
        (6003, 12.4881, 106.0188), // Kratie City
        (6004, 13.5259, 105.9683), // Stung Treng (Northern Mekong)
    ];

    let mut nr7_ids = vec![2003]; // Branch from Skun on NR6
    for (id, lat, lon) in nr7_nodes {
        builder.add_node(id, lat, lon);
        nr7_ids.push(id);
    }
    builder.add_way(&nr7_ids, false, 75.0);

    // -----------------------------------------------------------------------
    // National Road 1 (PP -> Neak Loeung Bridge -> Svay Rieng -> Bavet)
    // -----------------------------------------------------------------------
    let nr1_nodes = [
        (7001, 11.4500, 105.0500), // Kandal
        (7002, 11.2500, 105.2800), // Neak Loeung (Tsubasa Bridge)
        (7003, 11.0833, 105.8000), // Svay Rieng City
        (7004, 11.0750, 106.1500), // Bavet (Vietnam border)
    ];

    let mut nr1_ids = vec![1001];
    for (id, lat, lon) in nr1_nodes {
        builder.add_node(id, lat, lon);
        nr1_ids.push(id);
    }
    builder.add_way(&nr1_ids, false, 80.0);

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_demo_graph() {
        let graph = create_demo_graph();
        assert!(graph.node_count() >= 35, "Should have all major cities");
        assert!(
            graph.edge_count() >= 70,
            "Should have bidirectional corridors"
        );

        // Phnom Penh (node 0) to Siem Reap (node 2009)
        let pp_id = graph.get_internal_id(1001).expect("PP node exists");
        let sr_id = graph.get_internal_id(2009).expect("SR node exists");

        let route = crate::pathfinding::astar_search(&graph, pp_id, sr_id);
        assert!(route.is_some(), "Path between PP and Siem Reap must exist");
        let route = route.unwrap();
        let dist_km = route.distance_m / 1000.0;
        assert!(
            (250.0..=350.0).contains(&dist_km),
            "PP to SR should be ~260-320 km, got {dist_km:.1} km"
        );
    }

    #[test]
    fn test_phnom_penh_urban_routes() {
        let graph = create_demo_graph();

        // 1. Central Market Depot A (1010) -> Meanchey St 271 (1171)
        let depot_a = graph.get_internal_id(1010).expect("Depot A exists");
        let st271 = graph.get_internal_id(1171).expect("St 271 exists");
        let route1 = crate::pathfinding::astar_search(&graph, depot_a, st271);
        assert!(route1.is_some(), "Path from Depot A to St 271 must exist");
        let r1 = route1.unwrap();
        let dist1_km = r1.distance_m / 1000.0;
        assert!(
            (4.0..=8.0).contains(&dist1_km),
            "Depot A to St 271 should be ~4-8 km, got {dist1_km:.2} km"
        );
        assert!(r1.path.len() >= 4, "Should route via intermediate urban streets");

        // 2. Russian Market Depot B (1020) -> Tuol Kork St 289 (1200)
        let depot_b = graph.get_internal_id(1020).expect("Depot B exists");
        let tuol_kork = graph.get_internal_id(1200).expect("Tuol Kork exists");
        let route2 = crate::pathfinding::astar_search(&graph, depot_b, tuol_kork);
        assert!(route2.is_some(), "Path from Depot B to Tuol Kork must exist");
        let r2 = route2.unwrap();
        let dist2_km = r2.distance_m / 1000.0;
        assert!(
            (4.0..=8.0).contains(&dist2_km),
            "Depot B to Tuol Kork should be ~4-8 km, got {dist2_km:.2} km"
        );

        // 3. Bak Touk Hub (1030) -> Riverside (1121)
        let bak_touk = graph.get_internal_id(1030).expect("Bak Touk exists");
        let riverside = graph.get_internal_id(1121).expect("Riverside exists");
        let route3 = crate::pathfinding::astar_search(&graph, bak_touk, riverside);
        assert!(route3.is_some(), "Path from Bak Touk to Riverside must exist");
    }
}
