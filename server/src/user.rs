use crate::AppError;

use crate::auth::AuthUser;
use crate::public::ensure_player_exists;
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};

/// Query parameters for get_preds
#[derive(Deserialize)]
pub(crate) struct GetPredsQuery {
    bot: Option<String>,
}

/// Get predictions for a player (authenticated)
pub(crate) async fn get_preds(
    State(pool): State<SqlitePool>,
    Path(player_id): Path<i32>,
    Query(query): Query<GetPredsQuery>,
    Extension(auth_user): Extension<AuthUser>,
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

/// Save predictions
pub(crate) async fn save_preds(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<AuthUser>,
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

/// Clear my predictions (authenticated user)
pub(crate) async fn clear_my_preds(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<AuthUser>,
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

/// Get current user info from JWT token
#[derive(Serialize)]
pub(crate) struct MeResponse {
    player_id: i32,
    display_name: String,
    bot_name: Option<String>,
}

pub(crate) async fn get_current_user(
    State(pool): State<SqlitePool>,
    Extension(auth_user): Extension<AuthUser>,
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
