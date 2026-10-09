//! Algorithm comparison and benchmark test suite for P0-05.
//!
//! Validates that all supported pathfinding algorithms:
//! - Dijkstra
//! - A*
//! - Bidirectional Dijkstra
//! - Bidirectional A*
//! - Contraction Hierarchies
//! produce consistent, optimal route costs and valid geometry without synthetic fallbacks.

use osm_pathfinder::graph::create_demo_graph;
use osm_pathfinder::pathfinding::{
    astar_search_with_options, bidirectional_astar_search_with_options,
    bidirectional_dijkstra_search_with_options, build_contraction_hierarchies, ch_search,
    dijkstra_search_with_options, Algorithm, CostMetric, RoutingOptions, VehicleProfile,
};

#[test]
fn test_multi_algorithm_cost_equivalence() {
    let graph = create_demo_graph();
    let ch_graph = build_contraction_hierarchies(&graph);

    // Test OD pairs across Phnom Penh:
    // 1 -> 8 (Wat Phnom to Central Market)
    // 1 -> 35 (Wat Phnom to Royal Palace)
    // 8 -> 50 (Central Market to BKK1)
    let test_pairs = vec![(1, 8), (1, 35), (8, 50)];

    let options = RoutingOptions {
        metric: CostMetric::Distance,
        profile: VehicleProfile::Car,
        departure_minutes: None,
        collect_explored: false,
    };

    for &(start, end) in &test_pairs {
        let res_dijkstra = dijkstra_search_with_options(&graph, start, end, &options)
            .expect("Dijkstra must find path");
        let res_astar = astar_search_with_options(&graph, start, end, &options)
            .expect("A* must find path");
        let res_bi_dijkstra =
            bidirectional_dijkstra_search_with_options(&graph, start, end, &options)
                .expect("Bidirectional Dijkstra must find path");
        let res_bi_astar = bidirectional_astar_search_with_options(&graph, start, end, &options)
            .expect("Bidirectional A* must find path");
        let res_ch = ch_search(&ch_graph, &graph, start, end, &options)
            .expect("CH must find path");

        let optimal_dist = res_dijkstra.distance_m;

        // Ensure all algorithms agree within 0.1% or 1 meter
        assert!(
            (res_astar.distance_m - optimal_dist).abs() < 1.0,
            "A* distance {} differs from Dijkstra {}",
            res_astar.distance_m,
            optimal_dist
        );
        assert!(
            (res_bi_dijkstra.distance_m - optimal_dist).abs() < 1.0,
            "Bi-Dijkstra distance {} differs from Dijkstra {}",
            res_bi_dijkstra.distance_m,
            optimal_dist
        );
        assert!(
            (res_bi_astar.distance_m - optimal_dist).abs() < 1.0,
            "Bi-A* distance {} differs from Dijkstra {}",
            res_bi_astar.distance_m,
            optimal_dist
        );
        assert!(
            (res_ch.distance_m - optimal_dist).abs() < 1.0,
            "CH distance {} differs from Dijkstra {}",
            res_ch.distance_m,
            optimal_dist
        );

        // A* should explore <= nodes than Dijkstra
        assert!(
            res_astar.nodes_visited <= res_dijkstra.nodes_visited,
            "A* visited {} vs Dijkstra {}",
            res_astar.nodes_visited,
            res_dijkstra.nodes_visited
        );
    }
}

#[tokio::test]
async fn test_api_route_provenance_stamping() {
    let graph = create_demo_graph();
    let spatial_index = osm_pathfinder::spatial::SpatialIndex::new(&graph);
    let ch_graph = std::sync::Arc::new(build_contraction_hierarchies(&graph));
    let state = std::sync::Arc::new(osm_pathfinder::AppState::new_demo(
        graph,
        spatial_index,
        ch_graph,
    ));

    let req = osm_pathfinder::api::handlers::RouteRequest {
        start_lat: 11.5564,
        start_lon: 104.9282,
        end_lat: 11.5720,
        end_lon: 104.8980,
        algorithm: Algorithm::Astar,
        metric: CostMetric::Time,
        departure_time: None,
        profile: Some("car".to_string()),
        include_explored: false,
    };

    let result = osm_pathfinder::api::handlers::calculate_route(
        axum::extract::State(state.clone()),
        axum::Json(req),
    )
    .await;

    assert!(result.is_ok());
    let axum::Json(res) = result.unwrap();
    assert_eq!(res.is_demo, true);
    assert_eq!(res.graph_version, "demo-cambodia-v1.0");
    assert_eq!(res.cost_model_version, "tdsp-profiles-v1.0");
    assert!(res.distance_m > 0.0);
    assert!(res.duration_s > 0.0);
    assert!(!res.path.is_empty());

    // Stats endpoint provenance
    let stats = osm_pathfinder::api::handlers::graph_stats(axum::extract::State(state)).await;
    assert_eq!(stats.is_demo, true);
    assert_eq!(stats.graph_version, "demo-cambodia-v1.0");
    assert_eq!(stats.cost_model_version, "tdsp-profiles-v1.0");
    assert!(stats.nodes > 0);
    assert!(stats.edges > 0);
}
