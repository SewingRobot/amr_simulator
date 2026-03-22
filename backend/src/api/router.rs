use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware as axum_middleware,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::AppState;
use super::handlers::{assets, auth, maps, missions, robots, sim, tiles};
use super::middleware::auth::auth_middleware;
use super::ws::handler::ws_handler;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/login", post(auth::login))
        .route("/api/assets", get(assets::list_assets))
        .route("/api/assets/{id}/download", get(assets::download_asset));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/auth/me", get(auth::me))
        .route("/api/robots", get(robots::list_robots).post(robots::create_robot))
        .route(
            "/api/robots/{id}",
            get(robots::get_robot)
                .put(robots::update_robot)
                .delete(robots::delete_robot),
        )
        .route("/api/maps", get(maps::list_maps).post(maps::create_map))
        .route(
            "/api/maps/{id}",
            get(maps::get_map)
                .put(maps::update_map)
                .delete(maps::delete_map),
        )
        .route("/api/missions", get(missions::list_missions).post(missions::create_mission))
        .route(
            "/api/missions/{id}",
            get(missions::get_mission)
                .delete(missions::delete_mission),
        )
        .route("/api/missions/{id}/assign", post(missions::assign_mission))
        .route("/api/missions/{id}/cancel", post(missions::cancel_mission))
        .route("/api/sim/command", post(sim::send_command))
        .route("/api/maps/{map_id}/tiles/{node_id}", get(tiles::get_tile))
        .route("/api/maps/{map_id}/roadmap", get(tiles::get_roadmap))
        .route("/api/maps/{map_id}/roadmap/nodes", post(tiles::create_node))
        .route("/api/maps/{map_id}/roadmap/nodes/{node_id}", delete(tiles::delete_node))
        .route("/api/maps/{map_id}/roadmap/edges", post(tiles::create_edge))
        .route("/api/maps/{map_id}/roadmap/edges/{edge_id}", delete(tiles::delete_edge))
        .route("/api/maps/{map_id}/pathfind", post(tiles::find_path))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // WebSocket route (token validated inside handler)
    let ws_routes = Router::new().route("/api/ws", get(ws_handler));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(ws_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
