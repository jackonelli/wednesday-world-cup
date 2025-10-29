#![forbid(unsafe_code)]
use itertools::Itertools;
use structopt::StructOpt;
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupId};
use wwc_core::team::Team;
use wwc_data::lsv::LsvParseError;
use wwc_data::lsv::get_data;
use wwc_data::lsv::{Fifa2018Data, LsvData};

type Tournament = Fifa2018Data;
const DATA_PATH: &str = "data/lsv_data/blank-fifa-2018.json";

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let pool = wwc_db::create_pool().await?;
    let opt = Opt::from_args();
    match opt {
        Opt::Register(new_instance) => match new_instance {
            Register::Player { name } => register_player(&pool, name).await,
        },
        Opt::User(cmd) => match cmd {
            UserCommand::Create {
                username,
                password,
                player_id,
                display_name,
            } => create_user(&pool, player_id, username, password, display_name).await,
            UserCommand::List => list_users(&pool).await,
            UserCommand::Delete { username } => delete_user(&pool, username).await,
        },
        Opt::Bot(cmd) => match cmd {
            BotCommand::Create {
                username,
                bot_name,
                bot_display_name,
            } => create_bot(&pool, username, bot_name, bot_display_name).await,
            BotCommand::List { username } => list_bots(&pool, username).await,
            BotCommand::Delete { username, bot_name } => {
                delete_bot(&pool, username, bot_name).await
            }
        },
        Opt::Add(table) => match table {
            Table::Players => Ok(()),
            Table::Teams => add_teams(&pool).await,
            Table::Games => add_games(&pool).await,
            Table::GroupGameMaps => add_groups(&pool).await,
            Table::PlayoffTeamSources => add_playoff_team_sources(&pool).await,
            Table::All => {
                add_teams(&pool).await?;
                add_games(&pool).await?;
                add_groups(&pool).await?;
                add_playoff_team_sources(&pool).await
            }
        },
        Opt::List(table) => match table {
            Table::Players => list_players(&pool).await,
            Table::Teams => list_teams(&pool).await,
            Table::Games => list_games(&pool).await,
            Table::GroupGameMaps => list_group_maps(&pool).await,
            Table::PlayoffTeamSources => list_team_sources(&pool).await,
            Table::All => {
                list_players(&pool).await?;
                list_teams(&pool).await?;
                list_games(&pool).await?;
                list_group_maps(&pool).await?;
                list_team_sources(&pool).await
            }
        },
        Opt::Clear(table) => match table {
            Table::Players => Ok(wwc_db::clear_players(&pool).await?),
            Table::Teams => Ok(wwc_db::clear_teams(&pool).await?),
            Table::Games => Ok(wwc_db::clear_games(&pool).await?),
            Table::GroupGameMaps => Ok(wwc_db::clear_group_game_maps(&pool).await?),
            Table::PlayoffTeamSources => Ok(wwc_db::clear_playoff_team_sources(&pool).await?),
            Table::All => {
                // Clear child tables first to avoid foreign key constraints
                wwc_db::clear_preds(&pool).await?;
                wwc_db::clear_group_game_maps(&pool).await?;
                wwc_db::clear_playoff_team_sources(&pool).await?;
                wwc_db::clear_playoff_games(&pool).await?;
                wwc_db::clear_games(&pool).await?;
                wwc_db::clear_teams(&pool).await?;
                Ok(wwc_db::clear_players(&pool).await?)
            }
        },
    }
}

async fn register_player(pool: &sqlx::SqlitePool, name: String) -> Result<(), CliError> {
    Ok(wwc_db::register_player(pool, &name).await?)
}

async fn add_teams(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let teams = get_data::<Tournament>(DATA_PATH)?
        .try_teams()?
        .values()
        .cloned()
        .collect::<Vec<Team>>();
    Ok(wwc_db::insert_teams(pool, &teams).await?)
}

async fn add_games(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    // Add group games
    let groups = get_data::<Tournament>(DATA_PATH)?
        .try_groups()?
        .values()
        .cloned()
        .collect::<Vec<Group>>();

    let unplayed_games: Vec<_> = groups
        .iter()
        .flat_map(|group| group.unplayed_games())
        .cloned()
        .collect();
    wwc_db::insert_unplayed_games(pool, &unplayed_games).await?;
    let played_games: Vec<_> = groups
        .iter()
        .flat_map(|group| group.played_games())
        .cloned()
        .collect();
    wwc_db::insert_played_games(pool, &played_games).await?;

    // Add playoff game IDs (no teams/results yet - just the IDs)
    // The actual teams will be determined by BracketStructure from team_sources
    let team_sources = get_data::<Tournament>(DATA_PATH)?.team_sources;
    let playoff_game_ids: Vec<_> = team_sources.iter().map(|(game_id, _)| *game_id).collect();
    wwc_db::insert_playoff_games(pool, &playoff_game_ids).await?;
    Ok(())
}

async fn add_groups(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let groups = get_data::<Tournament>(DATA_PATH)?.try_groups()?;

    let group_games: Vec<(GroupId, GameId)> = groups
        .iter()
        .flat_map(move |(id, group)| {
            group
                .played_games()
                .map(move |game| (*id, game.id))
                .chain(group.unplayed_games().map(move |game| (*id, game.id)))
        })
        .collect();
    wwc_db::insert_group_game_mappings(pool, &group_games).await?;
    Ok(())
}

async fn add_playoff_team_sources(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let team_sources = get_data::<Tournament>(DATA_PATH)?.team_sources;
    wwc_db::insert_playoff_team_sources(pool, &team_sources).await?;
    Ok(())
}

async fn list_players(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    println!("PLAYERS:");
    let players = wwc_db::get_players(pool).await?;
    for player in players {
        println!("{:?}", player);
    }
    println!();
    Ok(())
}

// Auth management functions
async fn create_user(
    pool: &sqlx::SqlitePool,
    player_id: i32,
    username: String,
    password: String,
    display_name: String,
) -> Result<(), CliError> {
    // Ensure player exists (auto-create if needed)
    let player_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM players WHERE id = ?")
        .bind(player_id)
        .fetch_one(pool)
        .await
        .map_err(|e| CliError::Db(wwc_db::DbError::Sqlx(e)))?;

    if player_exists == 0 {
        // Create the player
        sqlx::query("INSERT INTO players (id, name) VALUES (?, ?)")
            .bind(player_id)
            .bind(&display_name)
            .execute(pool)
            .await
            .map_err(|e| CliError::Db(wwc_db::DbError::Sqlx(e)))?;
        println!("✓ Created player with ID {}", player_id);
    }

    let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|e| {
        CliError::Db(wwc_db::DbError::Generic(format!(
            "Failed to hash password: {}",
            e
        )))
    })?;

    wwc_db::create_user(pool, player_id, &username, &password_hash, &display_name).await?;
    println!("✓ Created user '{}'", username);
    println!("  Player ID: {}", player_id);
    println!("  Display Name: {}", display_name);
    Ok(())
}

async fn list_users(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let users = wwc_db::list_users(pool).await?;
    if users.is_empty() {
        println!("No users found");
    } else {
        println!("\nUsers:");
        println!(
            "{:<12} {:<20} {:<20}",
            "Player ID", "Username", "Display Name"
        );
        println!("{}", "-".repeat(55));
        for user in users {
            println!(
                "{:<12} {:<20} {:<20}",
                user.id, user.username, user.display_name
            );
        }
    }
    Ok(())
}

async fn delete_user(pool: &sqlx::SqlitePool, username: String) -> Result<(), CliError> {
    wwc_db::delete_user(pool, &username).await?;
    println!("✓ Deleted user '{}'", username);
    Ok(())
}

async fn create_bot(
    pool: &sqlx::SqlitePool,
    username: String,
    bot_name: String,
    bot_display_name: String,
) -> Result<(), CliError> {
    // Get user
    let user = wwc_db::get_user_by_username(pool, &username)
        .await?
        .ok_or_else(|| {
            CliError::Db(wwc_db::DbError::Generic(format!(
                "User '{}' not found",
                username
            )))
        })?;

    // Create bot record
    let bot_id = wwc_db::create_bot(pool, user.id, &bot_name, &bot_display_name).await?;

    // Generate JWT token for bot
    let token = generate_jwt_token(user.id, Some(bot_name.clone()));

    println!("✓ Created bot with ID {}", bot_id);
    println!("  Bot Name: {}", bot_name);
    println!("  Display Name: {}", bot_display_name);
    println!("  Player ID: {}", user.id);
    println!("\nToken (save this, it won't be shown again):");
    println!("{}", token);

    Ok(())
}

async fn list_bots(pool: &sqlx::SqlitePool, username: String) -> Result<(), CliError> {
    // Get user
    let user = wwc_db::get_user_by_username(pool, &username)
        .await?
        .ok_or_else(|| {
            CliError::Db(wwc_db::DbError::Generic(format!(
                "User '{}' not found",
                username
            )))
        })?;

    let bots = wwc_db::list_bots_for_user(pool, user.id).await?;
    if bots.is_empty() {
        println!("No bots found for user '{}'", username);
    } else {
        println!("\nBots for user '{}':", username);
        println!(
            "{:<5} {:<20} {:<30} {:<20}",
            "ID", "Bot Name", "Display Name", "Created At"
        );
        println!("{}", "-".repeat(80));
        for bot in bots {
            println!(
                "{:<5} {:<20} {:<30} {:<20}",
                bot.id, bot.bot_name, bot.bot_display_name, bot.created_at
            );
        }
    }
    Ok(())
}

async fn delete_bot(
    pool: &sqlx::SqlitePool,
    username: String,
    bot_name: String,
) -> Result<(), CliError> {
    // Get user
    let user = wwc_db::get_user_by_username(pool, &username)
        .await?
        .ok_or_else(|| {
            CliError::Db(wwc_db::DbError::Generic(format!(
                "User '{}' not found",
                username
            )))
        })?;

    wwc_db::delete_bot(pool, user.id, &bot_name).await?;
    println!("✓ Deleted bot '{}' for user '{}'", bot_name, username);
    Ok(())
}

fn generate_jwt_token(player_id: i32, bot_name: Option<String>) -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};

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

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("Warning: JWT_SECRET not set, using default (not secure for production)");
        "your-secret-key-change-this-in-production".to_string()
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate JWT token")
}

async fn list_teams(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let teams = wwc_db::get_teams(pool).await?;
    println!("TEAMS:");
    teams.iter().for_each(|team| println!("{}", team));
    println!();
    Ok(())
}

async fn list_games(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let games = wwc_db::get_games(pool).await?;
    println!("GAMES:");
    games.iter().for_each(|game| println!("{:?}", game));
    println!();
    Ok(())
}

async fn list_group_maps(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let group_game_maps = wwc_db::get_group_game_maps(pool).await?;
    println!("Group: Game mapping:");
    group_game_maps
        .into_iter()
        .map(|(game, group)| (group, game))
        .into_group_map()
        .iter()
        .for_each(|(group_id, games)| {
            println!(
                "{}: {}",
                group_id,
                games
                    .iter()
                    .fold(String::new(), |acc, x| format!("{} {},", acc, x))
            )
        });
    Ok(())
}

async fn list_team_sources(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let team_sources = wwc_db::get_playoff_team_sources(pool).await?;
    println!("Team sources");
    team_sources
        .into_iter()
        .for_each(|(game_id, (home_source, away_source))| {
            println!("{}: {}, {}", game_id, home_source, away_source)
        });
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wwc-cli", about = "cli usage")]
pub enum Opt {
    #[structopt(name = "register")]
    Register(Register),
    #[structopt(name = "add")]
    Add(Table),
    #[structopt(name = "list")]
    List(Table),
    #[structopt(name = "clear")]
    Clear(Table),
    #[structopt(name = "user")]
    User(UserCommand),
    #[structopt(name = "bot")]
    Bot(BotCommand),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wwc-cli-register", about = "Register new player")]
pub enum Register {
    #[structopt(name = "player")]
    Player { name: String },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wwc-cli-table", about = "Add table to database")]
pub enum Table {
    #[structopt(name = "players")]
    Players,
    #[structopt(name = "teams")]
    Teams,
    #[structopt(name = "games")]
    Games,
    #[structopt(name = "group-game-maps")]
    GroupGameMaps,
    #[structopt(name = "playoff-team-sources")]
    PlayoffTeamSources,
    #[structopt(name = "all")]
    All,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "user", about = "User management commands")]
pub enum UserCommand {
    #[structopt(name = "create")]
    Create {
        username: String,
        password: String,
        player_id: i32,
        display_name: String,
    },
    #[structopt(name = "list")]
    List,
    #[structopt(name = "delete")]
    Delete { username: String },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bot", about = "Bot management commands")]
pub enum BotCommand {
    #[structopt(name = "create")]
    Create {
        username: String,
        bot_name: String,
        bot_display_name: String,
    },
    #[structopt(name = "list")]
    List { username: String },
    #[structopt(name = "delete")]
    Delete { username: String, bot_name: String },
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Cli: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("Cli: {0}")]
    WwcCore(#[from] WwcError),
    #[error("Parse: {0}")]
    Parse(#[from] LsvParseError),
}
