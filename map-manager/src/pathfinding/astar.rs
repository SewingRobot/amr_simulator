use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

use super::graph::Graph;

/// Result of a pathfinding query.
#[derive(Debug, Clone)]
pub struct PathResult {
    /// Ordered list of node IDs from start to goal.
    pub node_ids: Vec<Uuid>,
    /// Total distance (cost) of the path.
    pub total_distance: f64,
    /// Estimated travel time in seconds (assuming 1 m/s default speed).
    pub estimated_time_s: f64,
}

/// A node in the A* priority queue.
#[derive(Debug, Clone)]
struct AStarNode {
    id: Uuid,
    f_score: f64,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score.total_cmp(&other.f_score) == Ordering::Equal
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior (BinaryHeap is a max-heap)
        other.f_score.total_cmp(&self.f_score)
    }
}

/// Euclidean distance heuristic in 3D space.
fn euclidean_distance(a: &(f64, f64, f64), b: &(f64, f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    let dz = a.2 - b.2;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Find the shortest path between two nodes using A* with Euclidean heuristic.
///
/// # Arguments
/// * `graph` - The graph to search
/// * `start` - Starting node ID
/// * `goal` - Goal node ID
/// * `positions` - Map of node IDs to (x, y, z) coordinates for the heuristic
///
/// # Returns
/// `Some(PathResult)` if a path exists, `None` otherwise.
pub fn find_path(
    graph: &Graph,
    start: Uuid,
    goal: Uuid,
    positions: &HashMap<Uuid, (f64, f64, f64)>,
) -> Option<PathResult> {
    // Early exit: start or goal not in graph
    if !graph.adjacency.contains_key(&start) || !graph.adjacency.contains_key(&goal) {
        return None;
    }

    // Early exit: start == goal
    if start == goal {
        return Some(PathResult {
            node_ids: vec![start],
            total_distance: 0.0,
            estimated_time_s: 0.0,
        });
    }

    let goal_pos = positions.get(&goal)?;

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<Uuid, Uuid> = HashMap::new();
    let mut g_score: HashMap<Uuid, f64> = HashMap::new();

    g_score.insert(start, 0.0);

    let start_h = positions
        .get(&start)
        .map_or(0.0, |pos| euclidean_distance(pos, goal_pos));

    open_set.push(AStarNode {
        id: start,
        f_score: start_h,
    });

    while let Some(current) = open_set.pop() {
        if current.id == goal {
            // Reconstruct path
            let mut path = vec![goal];
            let mut node = goal;
            while let Some(&prev) = came_from.get(&node) {
                path.push(prev);
                node = prev;
            }
            path.reverse();

            let total_distance = g_score[&goal];
            // Estimate time assuming average speed of 1.0 m/s
            let estimated_time_s = total_distance;

            return Some(PathResult {
                node_ids: path,
                total_distance,
                estimated_time_s,
            });
        }

        let current_g = g_score.get(&current.id).copied().unwrap_or(f64::INFINITY);

        // Skip if we already found a better path to this node
        if current.f_score > current_g + positions
            .get(&current.id)
            .map_or(0.0, |pos| euclidean_distance(pos, goal_pos))
            + f64::EPSILON * 10.0
        {
            continue;
        }

        for &(neighbor_id, edge_cost) in graph.neighbors(current.id) {
            let tentative_g = current_g + edge_cost;

            let current_neighbor_g = g_score.get(&neighbor_id).copied().unwrap_or(f64::INFINITY);

            if tentative_g < current_neighbor_g {
                came_from.insert(neighbor_id, current.id);
                g_score.insert(neighbor_id, tentative_g);

                let h = positions
                    .get(&neighbor_id)
                    .map_or(0.0, |pos| euclidean_distance(pos, goal_pos));

                open_set.push(AStarNode {
                    id: neighbor_id,
                    f_score: tentative_g + h,
                });
            }
        }
    }

    // No path found
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pathfinding::graph::Graph;

    #[test]
    fn test_find_path_simple() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(a, b, 1.0);
        graph.add_edge(b, c, 1.0);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 0.0, 0.0));
        positions.insert(c, (2.0, 0.0, 0.0));

        let result = find_path(&graph, a, c, &positions).unwrap();
        assert_eq!(result.node_ids, vec![a, b, c]);
        assert!((result.total_distance - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_no_path() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        // No edge connecting a and b

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 0.0, 0.0));

        let result = find_path(&graph, a, b, &positions);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_path_same_node() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        graph.add_node(a);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));

        let result = find_path(&graph, a, a, &positions).unwrap();
        assert_eq!(result.node_ids, vec![a]);
        assert!((result.total_distance - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_chooses_shortest() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);

        // Direct path a->c costs 10
        graph.add_edge(a, c, 10.0);
        // Indirect path a->b->c costs 3
        graph.add_edge(a, b, 1.0);
        graph.add_edge(b, c, 2.0);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 0.0, 0.0));
        positions.insert(c, (2.0, 0.0, 0.0));

        let result = find_path(&graph, a, c, &positions).unwrap();
        assert_eq!(result.node_ids, vec![a, b, c]);
        assert!((result.total_distance - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_bidirectional_edges() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);

        // Add bidirectional edges
        graph.add_edge(a, b, 1.0);
        graph.add_edge(b, a, 1.0);
        graph.add_edge(b, c, 1.0);
        graph.add_edge(c, b, 1.0);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 0.0, 0.0));
        positions.insert(c, (2.0, 0.0, 0.0));

        // Forward path a -> b -> c
        let fwd = find_path(&graph, a, c, &positions).unwrap();
        assert_eq!(fwd.node_ids, vec![a, b, c]);

        // Reverse path c -> b -> a (only works with bidirectional edges)
        let rev = find_path(&graph, c, a, &positions).unwrap();
        assert_eq!(rev.node_ids, vec![c, b, a]);
        assert!((fwd.total_distance - rev.total_distance).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_cost_factor_weighting() {
        // Simulate cost_factor: short-distance but high-cost edge vs longer but cheaper path
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);

        // Direct edge a->c with distance=2 * cost_factor=5 => effective cost 10
        graph.add_edge(a, c, 2.0 * 5.0);
        // Indirect a->b->c with lower effective cost: (1*1) + (1*1) = 2
        graph.add_edge(a, b, 1.0 * 1.0);
        graph.add_edge(b, c, 1.0 * 1.0);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 0.0, 0.0));
        positions.insert(c, (2.0, 0.0, 0.0));

        let result = find_path(&graph, a, c, &positions).unwrap();
        // Should prefer the cheaper indirect path
        assert_eq!(result.node_ids, vec![a, b, c]);
        assert!((result.total_distance - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_large_graph_performance() {
        // 100-node linear graph: 0 -> 1 -> 2 -> ... -> 99
        let node_ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();

        let mut graph = Graph::new();
        let mut positions = HashMap::new();

        for (i, &id) in node_ids.iter().enumerate() {
            graph.add_node(id);
            positions.insert(id, (i as f64, 0.0, 0.0));
        }

        for i in 0..99 {
            graph.add_edge(node_ids[i], node_ids[i + 1], 1.0);
        }

        let start = std::time::Instant::now();
        let result = find_path(&graph, node_ids[0], node_ids[99], &positions).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.node_ids.len(), 100);
        assert!((result.total_distance - 99.0).abs() < f64::EPSILON);
        // A* on a 100-node linear graph should complete well under 5ms
        assert!(
            elapsed.as_millis() < 5,
            "pathfinding took {}ms, expected < 5ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_find_path_large_graph_with_shortcut() {
        // 100-node graph with a shortcut from node 0 directly to node 99
        let node_ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();

        let mut graph = Graph::new();
        let mut positions = HashMap::new();

        for (i, &id) in node_ids.iter().enumerate() {
            graph.add_node(id);
            positions.insert(id, (i as f64, 0.0, 0.0));
        }

        for i in 0..99 {
            graph.add_edge(node_ids[i], node_ids[i + 1], 1.0);
        }
        // Add a direct shortcut with cost 50 (cheaper than 99 hops)
        graph.add_edge(node_ids[0], node_ids[99], 50.0);

        let result = find_path(&graph, node_ids[0], node_ids[99], &positions).unwrap();
        // Should take the shortcut (cost 50 < 99)
        assert_eq!(result.node_ids, vec![node_ids[0], node_ids[99]]);
        assert!((result.total_distance - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_path_node_not_in_graph() {
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let missing = Uuid::new_v4();

        graph.add_node(a);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(missing, (1.0, 0.0, 0.0));

        // Goal not in graph
        assert!(find_path(&graph, a, missing, &positions).is_none());
        // Start not in graph
        assert!(find_path(&graph, missing, a, &positions).is_none());
    }

    #[test]
    fn test_find_path_diamond_graph() {
        // Diamond: A -> B, A -> C, B -> D, C -> D
        // B path costs 3, C path costs 5 => should pick B
        let mut graph = Graph::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_node(d);

        graph.add_edge(a, b, 1.0);
        graph.add_edge(a, c, 3.0);
        graph.add_edge(b, d, 2.0);
        graph.add_edge(c, d, 2.0);

        let mut positions = HashMap::new();
        positions.insert(a, (0.0, 0.0, 0.0));
        positions.insert(b, (1.0, 1.0, 0.0));
        positions.insert(c, (1.0, -1.0, 0.0));
        positions.insert(d, (2.0, 0.0, 0.0));

        let result = find_path(&graph, a, d, &positions).unwrap();
        assert_eq!(result.node_ids, vec![a, b, d]);
        assert!((result.total_distance - 3.0).abs() < f64::EPSILON);
    }
}
