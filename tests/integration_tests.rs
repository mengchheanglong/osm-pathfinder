//! End-to-end integration tests for osm-pathfinder.

use osm_pathfinder::geo::haversine;
use osm_pathfinder::graph::GraphBuilder;
use osm_pathfinder::pathfinding::{astar_search, dijkstra_search, Algorithm};
use osm_pathfinder::spatial::SpatialIndex;

#[test]
fn test_end_to_end_routing_pipeline() {
    // 1. Build a synthetic network representing a city grid
    let mut builder = GraphBuilder::new();

    // Intersection 1: Downtown (Phnom Penh Central)
    let n1 = builder.add_node(1001, 11.5564, 104.9282);
    // Intersection 2: North
    let _n2 = builder.add_node(1002, 11.5664, 104.9282);
    // Intersection 3: East
    let _n3 = builder.add_node(1003, 11.5564, 104.9382);
    // Intersection 4: North-East destination
    let n4 = builder.add_node(1004, 11.5664, 104.9382);

    // Roads
    builder.add_way(&[1001, 1002], false, 50.0);
    builder.add_way(&[1002, 1004], false, 50.0);
    builder.add_way(&[1001, 1003], false, 50.0);
    builder.add_way(&[1003, 1004], false, 50.0);

    let graph = builder.build();
    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 8); // 4 bidirectional segments = 8 directed edges

    // 2. Spatial Index lookup
    let spatial_index = SpatialIndex::new(&graph);

    // Pick arbitrary GPS coordinates close to start and end
    let start_gps = (11.5560, 104.9280);
    let end_gps = (11.5668, 104.9380);

    let (start_node, _, _) = spatial_index
        .nearest_node(start_gps.0, start_gps.1)
        .expect("Should find start node");
    let (end_node, _, _) = spatial_index
        .nearest_node(end_gps.0, end_gps.1)
        .expect("Should find end node");

    assert_eq!(start_node, n1);
    assert_eq!(end_node, n4);

    // 3. Pathfinding with Dijkstra
    let dijkstra_res =
        dijkstra_search(&graph, start_node, end_node).expect("Dijkstra should find path");
    assert_eq!(dijkstra_res.algorithm, Algorithm::Dijkstra);
    assert_eq!(dijkstra_res.path.first(), Some(&start_node));
    assert_eq!(dijkstra_res.path.last(), Some(&end_node));
    assert!(dijkstra_res.distance_m > 0.0);

    // 4. Pathfinding with A*
    let astar_res = astar_search(&graph, start_node, end_node).expect("A* should find path");
    assert_eq!(astar_res.algorithm, Algorithm::Astar);
    assert_eq!(astar_res.path.first(), Some(&start_node));
    assert_eq!(astar_res.path.last(), Some(&end_node));

    // Both algorithms must find mathematically equal distance
    assert!(
        (dijkstra_res.distance_m - astar_res.distance_m).abs() < 1e-4,
        "Dijkstra and A* distances must match"
    );

    // 5. Pathfinding with Bidirectional A* & Bidirectional Dijkstra
    let bi_astar_res =
        osm_pathfinder::pathfinding::bidirectional_astar_search(&graph, start_node, end_node)
            .expect("Bidirectional A* should find path");
    assert_eq!(bi_astar_res.algorithm, Algorithm::BidirectionalAstar);
    assert_eq!(bi_astar_res.path.first(), Some(&start_node));
    assert_eq!(bi_astar_res.path.last(), Some(&end_node));
    assert!(
        (dijkstra_res.distance_m - bi_astar_res.distance_m).abs() < 1e-4,
        "Bidirectional A* distance must match Dijkstra"
    );

    let bi_dijkstra_res =
        osm_pathfinder::pathfinding::bidirectional_dijkstra_search(&graph, start_node, end_node)
            .expect("Bidirectional Dijkstra should find path");
    assert_eq!(bi_dijkstra_res.algorithm, Algorithm::BidirectionalDijkstra);
    assert!(
        (dijkstra_res.distance_m - bi_dijkstra_res.distance_m).abs() < 1e-4,
        "Bidirectional Dijkstra distance must match"
    );

    // 6. Pathfinding with Contraction Hierarchies
    let ch = osm_pathfinder::pathfinding::build_contraction_hierarchies(&graph);
    let ch_res = osm_pathfinder::pathfinding::ch_search(
        &ch,
        &graph,
        start_node,
        end_node,
        &osm_pathfinder::pathfinding::RoutingOptions::default(),
    )
    .expect("Contraction Hierarchies should find path");
    assert_eq!(ch_res.algorithm, Algorithm::ContractionHierarchies);
    assert_eq!(ch_res.path.first(), Some(&start_node));
    assert_eq!(ch_res.path.last(), Some(&end_node));
    assert!(
        (dijkstra_res.distance_m - ch_res.distance_m).abs() < 1e-4,
        "Contraction Hierarchies distance must match Dijkstra"
    );

    // 7. Verify coordinates in path
    assert_eq!(astar_res.coordinates.len(), astar_res.path.len());
    let direct_dist = haversine::distance(
        graph.get_coord(start_node).unwrap().lat,
        graph.get_coord(start_node).unwrap().lon,
        graph.get_coord(end_node).unwrap().lat,
        graph.get_coord(end_node).unwrap().lon,
    );
    assert!(
        astar_res.distance_m >= direct_dist,
        "Admissibility: road distance ({}) >= straight-line ({})",
        astar_res.distance_m,
        direct_dist
    );
}
