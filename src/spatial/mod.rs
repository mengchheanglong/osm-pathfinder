//! Spatial indexing for coordinate snapping.
//!
//! Provides an R-Tree index over graph node coordinates, enabling
//! O(log N) nearest-neighbor lookups to snap arbitrary GPS coordinates
//! to the nearest road graph node.

use rstar::{PointDistance, RTree, RTreeObject, AABB};

use crate::graph::RoadGraph;

/// A point in the R-Tree representing a graph node.
#[derive(Debug, Clone, Copy)]
struct IndexedPoint {
    /// Internal graph node ID.
    node_id: u32,
    /// Geographic coordinates.
    lat: f64,
    lon: f64,
}

impl RTreeObject for IndexedPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.lat, self.lon])
    }
}

impl PointDistance for IndexedPoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dlat = self.lat - point[0];
        let dlon = self.lon - point[1];
        dlat * dlat + dlon * dlon
    }
}

/// Spatial index for snapping coordinates to the nearest graph node.
///
/// Built from a [`RoadGraph`] at startup, this R-Tree provides fast
/// nearest-neighbor queries for arbitrary GPS coordinates.
pub struct SpatialIndex {
    tree: RTree<IndexedPoint>,
}

impl SpatialIndex {
    /// Builds a new spatial index from all nodes in the road graph.
    pub fn new(graph: &RoadGraph) -> Self {
        let points: Vec<IndexedPoint> = graph
            .coords
            .iter()
            .enumerate()
            .map(|(id, coord)| IndexedPoint {
                node_id: id as u32,
                lat: coord.lat,
                lon: coord.lon,
            })
            .collect();

        Self {
            tree: RTree::bulk_load(points),
        }
    }

    /// Finds the nearest graph node to the given coordinates.
    ///
    /// Returns `Some((node_id, lat, lon))` for the nearest node,
    /// or `None` if the index is empty.
    pub fn nearest_node(&self, lat: f64, lon: f64) -> Option<(u32, f64, f64)> {
        self.tree
            .nearest_neighbor(&[lat, lon])
            .map(|point| (point.node_id, point.lat, point.lon))
    }

    /// Returns the total number of indexed points.
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    /// Returns true if the index contains no points.
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphBuilder, RoadGraph};

    fn build_test_graph() -> RoadGraph {
        let mut builder = GraphBuilder::new();
        builder.add_node(1, 11.556, 104.928); // Node 0
        builder.add_node(2, 11.560, 104.930); // Node 1
        builder.add_node(3, 13.367, 103.845); // Node 2 (far away)
        builder.build()
    }

    #[test]
    fn test_nearest_node() {
        let graph = build_test_graph();
        let index = SpatialIndex::new(&graph);

        // Query near node 0
        let result = index.nearest_node(11.557, 104.929);
        assert!(result.is_some());
        let (id, _, _) = result.unwrap();
        assert_eq!(id, 0);
    }

    #[test]
    fn test_nearest_node_far() {
        let graph = build_test_graph();
        let index = SpatialIndex::new(&graph);

        // Query near Siem Reap (node 2)
        let result = index.nearest_node(13.370, 103.850);
        assert!(result.is_some());
        let (id, _, _) = result.unwrap();
        assert_eq!(id, 2);
    }

    #[test]
    fn test_index_size() {
        let graph = build_test_graph();
        let index = SpatialIndex::new(&graph);
        assert_eq!(index.len(), 3);
        assert!(!index.is_empty());
    }
}
