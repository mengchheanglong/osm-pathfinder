//! Real PBF ingestion and routing validation test on Cambodia OSM data.

use std::path::Path;
use osm_pathfinder::osm::parse_pbf;
use osm_pathfinder::pathfinding::astar_search;
use osm_pathfinder::spatial::SpatialIndex;

#[test]
fn test_real_cambodia_pbf_ingestion_and_routing() {
    let pbf_path = Path::new("data/cambodia-latest.osm.pbf");
    if !pbf_path.exists() {
        println!("Skipping real PBF test: data/cambodia-latest.osm.pbf not found");
        return;
    }

    println!("Parsing real Cambodia OSM PBF...");
    let start_parse = std::time::Instant::now();
    let graph = parse_pbf(pbf_path).expect("Real OSM PBF should parse successfully");
    let parse_duration = start_parse.elapsed();

    println!(
        "Parsed real Cambodia graph in {:?}: {} nodes, {} edges",
        parse_duration,
        graph.node_count(),
        graph.edge_count()
    );

    assert!(
        graph.node_count() > 1000,
        "Real road graph should have thousands of nodes, found {}",
        graph.node_count()
    );
    assert!(
        graph.edge_count() > 1000,
        "Real road graph should have thousands of edges, found {}",
        graph.edge_count()
    );

    // Build spatial index
    let spatial_index = SpatialIndex::new(&graph);

    // Test routing in central Phnom Penh: Independence Monument to Wat Phnom
    let pnh_monument = (11.5564, 104.9282);
    let wat_phnom = (11.5760, 104.9230);

    let (start_node, _, dist_start) = spatial_index
        .nearest_node(pnh_monument.0, pnh_monument.1)
        .expect("Should snap to start node");
    let (end_node, _, dist_end) = spatial_index
        .nearest_node(wat_phnom.0, wat_phnom.1)
        .expect("Should snap to end node");

    println!(
        "Snapped start within {:.1}m, end within {:.1}m",
        dist_start, dist_end
    );
    assert!(
        dist_start < 500.0,
        "Start snap distance should be reasonable in urban area"
    );
    assert!(
        dist_end < 500.0,
        "End snap distance should be reasonable in urban area"
    );

    // Run A*
    let astar_res = astar_search(&graph, start_node, end_node)
        .expect("A* route should be found on real Cambodia graph");

    println!(
        "A* route found: distance = {:.1}m, duration = {:.1}s, path points = {}",
        astar_res.distance_m,
        astar_res.duration_s,
        astar_res.path.len()
    );

    assert!(astar_res.distance_m > 500.0);
    assert!(astar_res.path.len() >= 2);
}
