# Database Layer - SQLx + SQLite

This crate provides async database operations using SQLx with SQLite.

## Setup from Scratch

### 1. Set Database URL

Create a `.env` file in the project root (or export the variable):

```bash
DATABASE_URL=sqlite:./server/data/wwc.db
```

Or for in-memory (testing):
```bash
DATABASE_URL=sqlite::memory:
```

### 2. Database Will Be Created Automatically

The database file and tables are created automatically when you first run:
- The server (`cargo run -p wwc_server`)
- The CLI (`cargo run -p wwc_cli`)
- Any code that calls `wwc_db::create_pool()`

The migration in `sqlx_migrations/001_create_tables.sql` runs automatically and is idempotent (safe to run multiple times).

### 3. Populate Data (Optional)

Use the CLI to add initial data:

```bash
# Add teams from data file
cargo run -p wwc_cli -- add teams

# Add games
cargo run -p wwc_cli -- add games

# Add group mappings
cargo run -p wwc_cli -- add group-game-maps

# Or add everything at once
cargo run -p wwc_cli -- add all

# Register a player
cargo run -p wwc_cli -- register player "Your Name"
```

### 4. View Data

```bash
# List all teams
cargo run -p wwc_cli -- list teams

# List all games
cargo run -p wwc_cli -- list games

# List everything
cargo run -p wwc_cli -- list all
```

### 5. Clear Data

```bash
# Clear all data
cargo run -p wwc_cli -- clear all

# Or clear specific tables
cargo run -p wwc_cli -- clear teams
cargo run -p wwc_cli -- clear games
```

## Migration from Diesel

If you have an existing Diesel database, it will work as-is! The schema is identical:
- Same table names
- Same column names
- Same data types

Just point `DATABASE_URL` to your existing SQLite file.

## Schema

The database has 5 tables:

1. **teams** - Team information (id, name, fifa_code, rank)
2. **games** - All games (id, type, teams, scores, played status)
3. **group_game_map** - Maps games to groups
4. **players** - Registered players
5. **preds** - Player predictions for games

See `sqlx_migrations/001_create_tables.sql` for the full schema.

## Development

The database layer is fully async and uses connection pooling:

```rust
use wwc_db::{create_pool, get_teams};

#[tokio::main]
async fn main() {
    // Create pool (runs migrations automatically)
    let pool = create_pool().await.unwrap();
    
    // Use it
    let teams = get_teams(&pool).await.unwrap();
    println!("Teams: {:?}", teams);
}
```

## Notes

- **Automatic migrations**: Tables are created automatically using `IF NOT EXISTS`
- **Idempotent**: Safe to call `create_pool()` multiple times
- **Connection pooling**: The `SqlitePool` manages connections efficiently
- **Async**: All operations are non-blocking
