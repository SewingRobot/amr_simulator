use sqlx::PgPool;

use crate::storage::s3_tiles::TileStorage;

/// MapService gRPC implementation.
///
/// This struct will implement the generated `MapService` trait from the proto
/// definition once proto compilation is enabled in build.rs.
///
/// # RPCs to implement:
///
/// ## Map CRUD
/// - `create_map` — Register map metadata (pre-upload step)
/// - `get_map` — Get map info by ID
/// - `list_maps` — List maps with pagination/filtering
/// - `update_map` — Update map metadata
/// - `delete_map` — Delete a map and associated data
///
/// ## Point Cloud Upload/Processing
/// - `get_upload_url` — Get presigned URL for point cloud upload
/// - `complete_upload` — Notify upload completion, start Potree tiling
/// - `get_processing_status` — Get processing status (pending/processing/completed/failed)
///
/// ## Tile Serving
/// - `get_tile` — Get Potree tile by map_id and node_name
/// - `stream_tiles` — Stream multiple tiles (for Sim Engine map loading)
/// - `get_potree_metadata` — Get Potree octree metadata
///
/// ## Roadmap Management
/// - `create_roadmap` — Create a roadmap for a map
/// - `get_roadmap` — Get a roadmap with all nodes and edges
/// - `list_roadmaps` — List roadmaps for a map
/// - `update_roadmap` — Update a roadmap (full replacement)
/// - `delete_roadmap` — Delete a roadmap
///
/// ## Roadmap Node/Edge Management
/// - `add_node` — Add a node to a roadmap
/// - `update_node` — Update a node
/// - `remove_node` — Remove a node (also removes connected edges)
/// - `add_edge` — Add an edge between two nodes
/// - `update_edge` — Update an edge
/// - `remove_edge` — Remove an edge
///
/// ## Semantic Regions
/// - `create_region` — Create a semantic region
/// - `get_region` — Get a region
/// - `list_regions` — List regions for a map
/// - `update_region` — Update a region
/// - `delete_region` — Delete a region
///
/// ## Pathfinding
/// - `find_path` — Find shortest path between two nodes (A*, Dijkstra)
///
/// ## Spatial Queries
/// - `find_nearest_node` — Find nearest node(s) to a coordinate
/// - `get_regions_at_point` — Find regions containing a point
#[allow(dead_code)]
pub struct MapServiceImpl {
    pool: PgPool,
    tile_storage: TileStorage,
}

#[allow(dead_code)]
impl MapServiceImpl {
    pub fn new(pool: PgPool, tile_storage: TileStorage) -> Self {
        Self { pool, tile_storage }
    }
}
