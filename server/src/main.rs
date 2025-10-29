mod admin;
mod auth;
mod err;
mod public;
mod user;

use err::AppError;

use crate::public::{get_display_names, get_groups, get_playoff_team_sources, get_teams};
use crate::user::{clear_my_preds, get_current_user, get_preds, save_preds};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    middleware,
    routing::{get, post, put},
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the database pool
    let pool = wwc_db::create_pool()
        .await
        .expect("Failed to create database pool");

    // Build our application with routes
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/get_teams", get(get_teams))
        .route("/get_groups", get(get_groups))
        .route("/get_playoff_team_sources", get(get_playoff_team_sources))
        .route("/get_display_names", get(get_display_names))
        .route("/login", post(auth::login));

    // User-authenticated routes (requires JWT token)
    let user_routes = Router::new()
        .route("/get_preds/:player_id", get(get_preds))
        .route("/save_preds", put(save_preds))
        .route("/clear_my_preds", get(clear_my_preds))
        .route("/me", get(get_current_user))
        .route_layer(middleware::from_fn(auth::user_auth_middleware));

    // Admin-only routes (requires ADMIN_SECRET)
    let admin_routes = Router::new()
        .route("/clear_preds", get(clear_preds))
        .route_layer(middleware::from_fn(auth::admin_auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ]),
        )
        .with_state(pool);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}

/// Clear all predictions (admin only)
async fn clear_preds(State(pool): State<SqlitePool>) -> Result<StatusCode, AppError> {
    wwc_db::clear_preds(&pool).await?;

    info!("All predictions cleared");
    Ok(StatusCode::OK)
}
