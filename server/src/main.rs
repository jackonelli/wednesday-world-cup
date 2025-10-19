use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use itertools::Itertools;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupId, Groups, game::PlayedGroupGame, game::UnplayedGroupGame};
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};
use wwc_core::team::Teams;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the database pool
    let pool = wwc_db::create_pool()
        .await
        .expect("Failed to create database pool");

    // Build our application with routes
    let app = Router::new()
        .route("/get_teams", get(get_teams))
        .route("/get_groups", get(get_groups))
        .route("/get_preds/:player_id", get(get_preds))
        .route("/clear_preds", get(clear_preds))
        .route("/save_preds", put(save_preds))
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

/// Save predictions
async fn save_preds(
    State(pool): State<SqlitePool>,
    Json(player_preds): Json<PlayerPredictions>,
) -> Result<StatusCode, AppError> {
    info!("Saving predictions for player {}", player_preds.id);

    // Ensure player exists (auto-create if needed)
    ensure_player_exists(&pool, player_preds.id).await?;

    wwc_db::insert_preds(&pool, &player_preds).await?;

    Ok(StatusCode::OK)
}

/// Get teams
async fn get_teams(State(pool): State<SqlitePool>) -> Result<Json<Teams>, AppError> {
    let teams: Teams = wwc_db::get_teams(&pool)
        .await?
        .into_iter()
        .map(|x| (x.id, x))
        .collect();

    info!("Retrieved {} teams", teams.len());
    Ok(Json(teams))
}

/// Get predictions for a player
async fn get_preds(
    State(pool): State<SqlitePool>,
    Path(player_id): Path<i32>,
) -> Result<Json<Vec<Prediction>>, AppError> {
    let preds = wwc_db::get_preds(&pool, PlayerId::from(player_id)).await?;

    info!(
        "Retrieved {} predictions for player {}",
        preds.len(),
        player_id
    );
    Ok(Json(preds))
}

/// Clear all predictions
async fn clear_preds(State(pool): State<SqlitePool>) -> Result<StatusCode, AppError> {
    wwc_db::clear_preds(&pool).await?;

    info!("Predictions cleared");
    Ok(StatusCode::OK)
}

/// Get groups
///
/// Loads group games and a GameId: GroupId map from the db.
/// The games (played and unplayed) are then mapped to prospective groups.
/// The final groups are validated (with a fallible constructor) and collected together.
async fn get_groups(State(pool): State<SqlitePool>) -> Result<Json<Groups>, AppError> {
    let (played_games, unplayed_games) = wwc_db::get_group_games(&pool).await?;

    let game_group_map = wwc_db::get_group_game_maps(&pool)
        .await?
        .into_iter()
        .collect::<HashMap<GameId, GroupId>>();

    let empty_groups = game_group_map
        .iter()
        .map(|(_game_id, group_id)| group_id)
        .unique();

    let groups_played =
        played_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<PlayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups_unplayed =
        unplayed_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<UnplayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups: Result<Groups, WwcError> = groups_played
        .into_iter()
        .zip(groups_unplayed.into_iter())
        .map(
            |((group_id_played, played), (group_id_unplayed, unplayed))| {
                assert!(group_id_played == group_id_unplayed);
                Group::try_new(unplayed, played)
                    .map(|group| (group_id_played, group))
                    .map_err(WwcError::from)
            },
        )
        .collect();

    let groups = groups?;
    info!("Retrieved {} groups", groups.len());
    Ok(Json(groups))
}

/// Helper function to ensure a player exists in the database
/// Creates a default player with the given ID if it doesn't exist
async fn ensure_player_exists(pool: &SqlitePool, player_id: PlayerId) -> Result<(), AppError> {
    // Check if player exists
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM players WHERE id = ?")
        .bind(i32::from(player_id))
        .fetch_one(pool)
        .await?;

    // If player doesn't exist, create them
    if exists == 0 {
        let player_name = format!("Player {}", i32::from(player_id));
        sqlx::query("INSERT INTO players (id, name) VALUES (?, ?)")
            .bind(i32::from(player_id))
            .bind(player_name)
            .execute(pool)
            .await?;
        info!("Auto-created player with id {}", player_id);
    }

    Ok(())
}

// Error handling
#[derive(Error, Debug)]
enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("WWC core error: {0}")]
    Wwc(#[from] WwcError),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Wwc(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Sqlx(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, error_message).into_response()
    }
}
