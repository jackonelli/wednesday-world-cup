// Core modules
mod models;
mod pool;

// Shared operations
mod games;
mod team;

// Domain-specific operations
mod auth;
mod group;
mod player;
mod playoff;

use thiserror::Error;
use wwc_core::error::WwcError;

// Re-export all public functions
pub use auth::{
    create_bot, create_user, delete_bot, delete_user, get_all_display_names, get_bot,
    get_user_by_id, get_user_by_username, list_bots_for_user, list_users,
};
pub use games::{clear_games, get_games, insert_played_games, insert_unplayed_games};
pub use group::{
    clear_group_game_maps, get_group_game_maps, get_group_games, insert_group_game_mappings,
};
pub use player::{
    clear_player_preds, clear_players, clear_preds, get_players, get_preds, insert_preds,
    register_player,
};
pub use playoff::{
    clear_playoff_games, clear_playoff_team_sources, get_playoff_team_sources,
    insert_playoff_games, insert_playoff_team_sources,
};
pub use pool::create_pool;
pub use team::{clear_teams, get_teams, insert_teams};

// Re-export models that are used in public APIs
pub use models::{Bot, Game, Player, User};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Missing 'DATABASE_URL'")]
    DbUrlMissing,
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Core error: {0}")]
    Core(#[from] WwcError),
    #[error("Could you be more specific: {0}")]
    Generic(String),
}
