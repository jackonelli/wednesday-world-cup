mod auth;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
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
use wwc_core::playoff::TeamSource;
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
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/get_teams", get(get_teams))
        .route("/get_groups", get(get_groups))
        .route("/get_playoff_team_sources", get(get_playoff_team_sources))
        .route("/login", post(login))
        .route("/get_display_names", get(get_display_names))
        .route("/get_preds/:player_id", get(get_preds_public));

    // User-authenticated routes (requires JWT token)
    let user_routes = Router::new()
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

/// Save predictions
async fn save_preds(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<auth::AuthUser>,
    Json(player_preds): Json<PlayerPredictions>,
) -> Result<StatusCode, AppError> {
    // Verify the player_id in the predictions matches the authenticated user's player_id
    if i32::from(player_preds.id) != auth_user.player_id {
        return Err(AppError::Generic(
            "Cannot save predictions for a different player".to_string(),
        ));
    }

    info!(
        "Saving predictions for player {} (bot: {:?})",
        player_preds.id, auth_user.bot_name
    );

    // Ensure player exists (auto-create if needed)
    ensure_player_exists(&pool, player_preds.id).await?;

    wwc_db::insert_preds(&pool, &player_preds, auth_user.bot_name.as_deref()).await?;

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

/// Query parameters for get_preds
#[derive(Deserialize)]
struct GetPredsQuery {
    bot: Option<String>,
}

/// Get predictions for a player (public, no auth required - temporary)
async fn get_preds_public(
    State(pool): State<SqlitePool>,
    Path(player_id): Path<i32>,
    Query(query): Query<GetPredsQuery>,
) -> Result<Json<Vec<Prediction>>, AppError> {
    let bot_name = query.bot.as_deref();
    let preds = wwc_db::get_preds(&pool, PlayerId::from(player_id), bot_name).await?;

    info!(
        "Retrieved {} predictions for player {} (bot: {:?})",
        preds.len(),
        player_id,
        bot_name
    );
    Ok(Json(preds))
}

/// Get predictions for a player (authenticated - for future use when UI is updated)
#[allow(dead_code)]
async fn get_preds(
    State(pool): State<SqlitePool>,
    Path(player_id): Path<i32>,
    Query(query): Query<GetPredsQuery>,
    Extension(auth_user): Extension<auth::AuthUser>,
) -> Result<Json<Vec<Prediction>>, AppError> {
    // Verify the requested player_id matches the authenticated user's player_id
    if player_id != auth_user.player_id {
        return Err(AppError::Generic(
            "Cannot access predictions for a different player".to_string(),
        ));
    }

    let bot_name = query.bot.as_deref();
    let preds = wwc_db::get_preds(&pool, PlayerId::from(player_id), bot_name).await?;

    info!(
        "Retrieved {} predictions for player {} (bot: {:?})",
        preds.len(),
        player_id,
        bot_name
    );
    Ok(Json(preds))
}

/// Clear all predictions (admin only)
async fn clear_preds(State(pool): State<SqlitePool>) -> Result<StatusCode, AppError> {
    wwc_db::clear_preds(&pool).await?;

    info!("All predictions cleared");
    Ok(StatusCode::OK)
}

/// Clear my predictions (authenticated user)
async fn clear_my_preds(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<auth::AuthUser>,
) -> Result<StatusCode, AppError> {
    wwc_db::clear_player_preds(
        &pool,
        PlayerId::from(auth_user.player_id),
        auth_user.bot_name.as_deref(),
    )
    .await?;

    info!(
        "Cleared predictions for player {} (bot: {:?})",
        auth_user.player_id, auth_user.bot_name
    );
    Ok(StatusCode::OK)
}

/// Login request
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Login response
#[derive(Serialize)]
struct LoginResponse {
    token: String,
    player_id: i32,
}

/// Login endpoint
async fn login(
    State(pool): State<SqlitePool>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Get user by username
    let user = wwc_db::get_user_by_username(&pool, &login_req.username)
        .await?
        .ok_or_else(|| AppError::Generic("Invalid username or password".to_string()))?;

    // Verify password
    let password_valid = bcrypt::verify(&login_req.password, &user.password_hash)
        .map_err(|_| AppError::Generic("Invalid username or password".to_string()))?;

    if !password_valid {
        return Err(AppError::Generic(
            "Invalid username or password".to_string(),
        ));
    }

    // Generate JWT token (for human user, not a bot)
    let token = generate_jwt_token(user.id, None);

    info!("User '{}' logged in", user.username);

    Ok(Json(LoginResponse {
        token,
        player_id: user.id,
    }))
}

/// Get current user info from JWT token
#[derive(Serialize)]
struct MeResponse {
    player_id: i32,
    display_name: String,
    bot_name: Option<String>,
}

async fn get_current_user(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<auth::AuthUser>,
) -> Result<Json<MeResponse>, AppError> {
    // Get user from database
    let user = wwc_db::get_user_by_id(&pool, auth_user.player_id)
        .await?
        .ok_or_else(|| AppError::Generic("User not found".to_string()))?;

    info!("User {} fetched their info", auth_user.player_id);

    Ok(Json(MeResponse {
        player_id: auth_user.player_id,
        display_name: user.display_name,
        bot_name: auth_user.bot_name,
    }))
}

/// Get all display names
async fn get_display_names(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<(i32, Option<String>, String)>>, AppError> {
    let display_names = wwc_db::get_all_display_names(&pool).await?;

    info!("Retrieved {} display names", display_names.len());
    Ok(Json(display_names))
}

/// Generate a JWT token
fn generate_jwt_token(player_id: i32, bot_name: Option<String>) -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        player_id: i32,
        bot_name: Option<String>,
        exp: usize,
    }

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        player_id,
        bot_name,
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate JWT token")
}

/// Get playoff team sources
///
/// Returns the team sources for all playoff games.
/// This is used to build the BracketStructure on the client.
async fn get_playoff_team_sources(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<(GameId, (TeamSource, TeamSource))>>, AppError> {
    let team_sources = wwc_db::get_playoff_team_sources(&pool).await?;

    info!("Retrieved {} playoff team sources", team_sources.len());
    Ok(Json(team_sources))
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
    #[error("{0}")]
    Generic(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        let (status, error_message) = match self {
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Wwc(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Sqlx(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Generic(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let error_response = ErrorResponse {
            error: error_message,
        };

        (status, Json(error_response)).into_response()
    }
}
