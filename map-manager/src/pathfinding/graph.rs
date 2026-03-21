use std::collections::HashMap;
use uuid::Uuid;

use crate::models::roadmap::{RoadmapEdgeRecord, RoadmapNodeRecord};

/// Graph data structure using adjacency list representation.
///
/// Each node maps to a list of (neighbor_id, edge_cost) tuples.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// Adjacency list: node_id -> [(neighbor_id, cost)]
    pub adjacency: HashMap<Uuid, Vec<(Uuid, f64)>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Add a node to the graph (creates an empty adjacency entry).
    pub fn add_node(&mut self, node_id: Uuid) {
        self.adjacency.entry(node_id).or_default();
    }

    /// Add a directed edge from `source` to `target` with the given cost.
    pub fn add_edge(&mut self, source: Uuid, target: Uuid, cost: f64) {
        self.adjacency
            .entry(source)
            .or_default()
            .push((target, cost));
    }

    /// Get neighbors and edge costs for a node.
    pub fn neighbors(&self, node_id: Uuid) -> &[(Uuid, f64)] {
        self.adjacency.get(&node_id).map_or(&[], |v| v.as_slice())
    }

    /// Build a graph from database records.
    ///
    /// For bidirectional edges, both directions are added.
    /// Edge cost = distance * cost_factor.
    pub fn build_from_records(
        nodes: &[RoadmapNodeRecord],
        edges: &[RoadmapEdgeRecord],
    ) -> Self {
        let mut graph = Self::new();

        for node in nodes {
            graph.add_node(node.id);
        }

        for edge in edges {
            let cost = edge.distance * edge.cost_factor;
            graph.add_edge(edge.source_node_id, edge.target_node_id, cost);

            // Add reverse edge for bidirectional edges
            if edge.direction == "bi" {
                graph.add_edge(edge.target_node_id, edge.source_node_id, cost);
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_basic_operations() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(a, b, 1.0);
        graph.add_edge(b, c, 2.0);

        assert_eq!(graph.neighbors(a).len(), 1);
        assert_eq!(graph.neighbors(b).len(), 1);
        assert_eq!(graph.neighbors(c).len(), 0);
    }
}
