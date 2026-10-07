//! Benchmarks for pathfinding algorithms.
//!
//! Run with: `cargo bench`
//!
//! These benchmarks use Criterion for statistical analysis.
//! They operate on small synthetic graphs — for real-world performance
//! testing, use the full OSM dataset.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_grid_graph(size: usize) -> osm_pathfinder::graph::RoadGraph {
    // Note: This requires making graph types public in lib.
    // For now, this is a placeholder that will be filled in
    // once the crate is structured as a library + binary.
    todo!("Implement grid graph generation for benchmarks")
}

fn bench_dijkstra(c: &mut Criterion) {
    c.bench_function("dijkstra_placeholder", |b| {
        b.iter(|| {
            // Placeholder: will benchmark actual algorithm once
            // the crate lib interface is finalized
            black_box(42)
        });
    });
}

fn bench_astar(c: &mut Criterion) {
    c.bench_function("astar_placeholder", |b| {
        b.iter(|| {
            black_box(42)
        });
    });
}

criterion_group!(benches, bench_dijkstra, bench_astar);
criterion_main!(benches);
