use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::models::{RoadmapEdge, RoadmapNode};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Response / request types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RoadmapResponse {
    pub map_id: Uuid,
    pub nodes: Vec<RoadmapNode>,
    pub edges: Vec<RoadmapEdge>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub name: Option<String>,
    #[serde(default = "default_node_type")]
    pub node_type: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

fn default_node_type() -> String {
    "waypoint".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    #[serde(default = "default_max_speed")]
    pub max_speed: f64,
    #[serde(default = "default_direction")]
    pub direction: String,
}

fn default_max_speed() -> f64 {
    1.0
}

fn default_direction() -> String {
    "bi".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PathfindRequest {
    pub start_node_id: Uuid,
    pub end_node_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PathResult {
    pub node_ids: Vec<Uuid>,
    pub total_distance: f64,
    pub estimated_time_s: f64,
}

// ---------------------------------------------------------------------------
// Tile handler (unchanged)
// ---------------------------------------------------------------------------

/// GET /api/maps/:map_id/tiles/:node_id -- point cloud tile proxy.
/// For Phase 1: returns 404 (point cloud tiles not yet available).
pub async fn get_tile(
    Path((map_id, node_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::NotFound(format!(
        "Tile {}/{} not yet available — point cloud tiles are not implemented in Phase 1",
        map_id, node_id
    )))
}

// ---------------------------------------------------------------------------
// Roadmap CRUD
// ---------------------------------------------------------------------------

/// GET /api/maps/:map_id/roadmap
pub async fn get_roadmap(
    State(state): State<AppState>,
    Path(map_id): Path<Uuid>,
) -> Result<Json<RoadmapResponse>, AppError> {
    let nodes: Vec<RoadmapNode> = sqlx::query_as(
        "SELECT id, map_id, name, node_type, x, y, z, properties, created_at \
         FROM roadmap_nodes WHERE map_id = $1 ORDER BY created_at",
    )
    .bind(map_id)
    .fetch_all(&state.pool)
    .await?;

    let edges: Vec<RoadmapEdge> = sqlx::query_as(
        "SELECT id, map_id, source_node_id, target_node_id, distance, max_speed, \
         direction, cost_factor, properties, created_at \
         FROM roadmap_edges WHERE map_id = $1 ORDER BY created_at",
    )
    .bind(map_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(RoadmapResponse {
        map_id,
        nodes,
        edges,
    }))
}

/// POST /api/maps/:map_id/roadmap/nodes
pub async fn create_node(
    State(state): State<AppState>,
    Path(map_id): Path<Uuid>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<Json<RoadmapNode>, AppError> {
    let node: RoadmapNode = sqlx::query_as(
        r#"INSERT INTO roadmap_nodes (map_id, name, node_type, x, y, z)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, map_id, name, node_type, x, y, z, properties, created_at"#,
    )
    .bind(map_id)
    .bind(&payload.name)
    .bind(&payload.node_type)
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.z)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(node))
}

/// DELETE /api/maps/:map_id/roadmap/nodes/:node_id
pub async fn delete_node(
    State(state): State<AppState>,
    Path((_map_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM roadmap_nodes WHERE id = $1")
        .bind(node_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Node {} not found", node_id)));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": node_id.to_string() })))
}

/// POST /api/maps/:map_id/roadmap/edges
pub async fn create_edge(
    State(state): State<AppState>,
    Path(map_id): Path<Uuid>,
    Json(payload): Json<CreateEdgeRequest>,
) -> Result<Json<RoadmapEdge>, AppError> {
    // Look up source and target node positions to auto-calculate distance
    let source: RoadmapNode = sqlx::query_as(
        "SELECT id, map_id, name, node_type, x, y, z, properties, created_at \
         FROM roadmap_nodes WHERE id = $1",
    )
    .bind(payload.source_node_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!("Source node {} not found", payload.source_node_id))
    })?;

    let target: RoadmapNode = sqlx::query_as(
        "SELECT id, map_id, name, node_type, x, y, z, properties, created_at \
         FROM roadmap_nodes WHERE id = $1",
    )
    .bind(payload.target_node_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!("Target node {} not found", payload.target_node_id))
    })?;

    let dx = source.x - target.x;
    let dy = source.y - target.y;
    let dz = source.z - target.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    let edge: RoadmapEdge = sqlx::query_as(
        r#"INSERT INTO roadmap_edges (map_id, source_node_id, target_node_id, distance, max_speed, direction)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, map_id, source_node_id, target_node_id, distance, max_speed, direction, cost_factor, properties, created_at"#,
    )
    .bind(map_id)
    .bind(payload.source_node_id)
    .bind(payload.target_node_id)
    .bind(distance)
    .bind(payload.max_speed)
    .bind(&payload.direction)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(edge))
}

/// DELETE /api/maps/:map_id/roadmap/edges/:edge_id
pub async fn delete_edge(
    State(state): State<AppState>,
    Path((_map_id, edge_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM roadmap_edges WHERE id = $1")
        .bind(edge_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Edge {} not found", edge_id)));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": edge_id.to_string() })))
}

// ---------------------------------------------------------------------------
// A* pathfinding endpoint
// ---------------------------------------------------------------------------

/// POST /api/maps/:map_id/pathfind
pub async fn find_path(
    State(state): State<AppState>,
    Path(map_id): Path<Uuid>,
    Json(req): Json<PathfindRequest>,
) -> Result<Json<PathResult>, AppError> {
    // Load nodes and edges from DB
    let nodes: Vec<RoadmapNode> = sqlx::query_as(
        "SELECT id, map_id, name, node_type, x, y, z, properties, created_at \
         FROM roadmap_nodes WHERE map_id = $1",
    )
    .bind(map_id)
    .fetch_all(&state.pool)
    .await?;

    let edges: Vec<RoadmapEdge> = sqlx::query_as(
        "SELECT id, map_id, source_node_id, target_node_id, distance, max_speed, \
         direction, cost_factor, properties, created_at \
         FROM roadmap_edges WHERE map_id = $1",
    )
    .bind(map_id)
    .fetch_all(&state.pool)
    .await?;

    // Build positions map
    let mut positions: HashMap<Uuid, (f64, f64, f64)> = HashMap::new();
    for node in &nodes {
        positions.insert(node.id, (node.x, node.y, node.z));
    }

    // Build adjacency list
    let mut adjacency: HashMap<Uuid, Vec<(Uuid, f64)>> = HashMap::new();
    for node in &nodes {
        adjacency.entry(node.id).or_default();
    }
    for edge in &edges {
        let cost = edge.distance * edge.cost_factor;
        adjacency
            .entry(edge.source_node_id)
            .or_default()
            .push((edge.target_node_id, cost));
        if edge.direction == "bi" {
            adjacency
                .entry(edge.target_node_id)
                .or_default()
                .push((edge.source_node_id, cost));
        }
    }

    // Run A*
    let result = astar(
        &adjacency,
        &positions,
        req.start_node_id,
        req.end_node_id,
    )
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "No path found from {} to {}",
            req.start_node_id, req.end_node_id
        ))
    })?;

    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Inline A* implementation
// ---------------------------------------------------------------------------

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
        // Reverse for min-heap
        other.f_score.total_cmp(&self.f_score)
    }
}

fn euclidean_distance(a: &(f64, f64, f64), b: &(f64, f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    let dz = a.2 - b.2;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn astar(
    adjacency: &HashMap<Uuid, Vec<(Uuid, f64)>>,
    positions: &HashMap<Uuid, (f64, f64, f64)>,
    start: Uuid,
    goal: Uuid,
) -> Option<PathResult> {
    if !adjacency.contains_key(&start) || !adjacency.contains_key(&goal) {
        return None;
    }

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
            let mut path = vec![goal];
            let mut node = goal;
            while let Some(&prev) = came_from.get(&node) {
                path.push(prev);
                node = prev;
            }
            path.reverse();

            let total_distance = g_score[&goal];
            let estimated_time_s = total_distance; // 1 m/s default

            return Some(PathResult {
                node_ids: path,
                total_distance,
                estimated_time_s,
            });
        }

        let current_g = g_score.get(&current.id).copied().unwrap_or(f64::INFINITY);

        if current.f_score
            > current_g
                + positions
                    .get(&current.id)
                    .map_or(0.0, |pos| euclidean_distance(pos, goal_pos))
                + f64::EPSILON * 10.0
        {
            continue;
        }

        if let Some(neighbors) = adjacency.get(&current.id) {
            for &(neighbor_id, edge_cost) in neighbors {
                let tentative_g = current_g + edge_cost;
                let current_neighbor_g =
                    g_score.get(&neighbor_id).copied().unwrap_or(f64::INFINITY);

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
    }

    None
}
