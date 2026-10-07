//! Demo graph generator for out-of-the-box exploration without a large PBF file.
//!
//! Generates a realistic road network connecting major Cambodian cities,
//! national highway corridors (NR1, NR3, NR4, NR5, NR6, NR7, Expressway),
//! and urban street grids in Phnom Penh and Siem Reap.

use super::builder::GraphBuilder;
use super::types::RoadGraph;

/// Creates a demo road network spanning Cambodia with national highway corridors.
pub fn create_demo_graph() -> RoadGraph {
    let mut builder = GraphBuilder::new();

    // -----------------------------------------------------------------------
    // Major Cities & Junctions
    // -----------------------------------------------------------------------
    // Phnom Penh Center
    builder.add_node(1001, 11.5564, 104.9282); // Phnom Penh Central
    builder.add_node(1002, 11.5760, 104.9230); // PP North (Wat Phnom)
    builder.add_node(1003, 11.5450, 104.9310); // PP South (Chamkarmon)
    builder.add_node(1004, 11.5600, 104.8900); // PP West (Toul Kork / Airport Exit)

    // PP Urban Street Grid
    builder.add_way(&[1001, 1002], false, 40.0);
    builder.add_way(&[1001, 1003], false, 40.0);
    builder.add_way(&[1001, 1004], false, 40.0);
    builder.add_way(&[1002, 1004], false, 40.0);
    builder.add_way(&[1003, 1004], false, 40.0);

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
    builder.add_way(&[1004, 4101, 4102, 4103, 4004], false, 120.0);

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
}
