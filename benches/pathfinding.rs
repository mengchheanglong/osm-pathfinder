//! Benchmarks for pathfinding algorithms.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use osm_pathfinder::graph::{GraphBuilder, RoadGraph};
use osm_pathfinder::pathfinding::{
    astar_search, bidirectional_astar_search, bidirectional_dijkstra_search, dijkstra_search,
};

/// Generates a synthetic NxN grid graph for benchmarking.
fn create_grid_graph(size: usize) -> RoadGraph {
    let mut builder = GraphBuilder::with_capacity(size * size);

    for i in 0..size {
        for j in 0..size {
            let osm_id = (i * size + j + 1) as i64;
            let lat = 11.0 + (i as f64) * 0.005;
            let lon = 104.0 + (j as f64) * 0.005;
            builder.add_node(osm_id, lat, lon);
        }
    }

    for i in 0..size {
        for j in 0..size {
            let id = (i * size + j + 1) as i64;
            if j + 1 < size {
                builder.add_way(&[id, id + 1], false, 50.0);
            }
            if i + 1 < size {
                builder.add_way(&[id, id + size as i64], false, 50.0);
            }
        }
    }

    builder.build()
}

fn bench_pathfinding(c: &mut Criterion) {
    let grid_size = 20; // 400 nodes
    let graph = create_grid_graph(grid_size);
    let start_node = 0;
    let end_node = (grid_size * grid_size - 1) as u32;

    let mut group = c.benchmark_group("pathfinding_20x20_grid");

    group.bench_function("dijkstra", |b| {
        b.iter(|| {
            dijkstra_search(black_box(&graph), black_box(start_node), black_box(end_node))
        });
    });

    group.bench_function("astar", |b| {
        b.iter(|| {
            astar_search(black_box(&graph), black_box(start_node), black_box(end_node))
        });
    });

    group.bench_function("bidirectional_dijkstra", |b| {
        b.iter(|| {
            bidirectional_dijkstra_search(black_box(&graph), black_box(start_node), black_box(end_node))
        });
    });

    group.bench_function("bidirectional_astar", |b| {
        b.iter(|| {
            bidirectional_astar_search(black_box(&graph), black_box(start_node), black_box(end_node))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_pathfinding);
criterion_main!(benches);
