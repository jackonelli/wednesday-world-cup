-- Note: bot_name column for preds table should be added manually if upgrading existing DB
-- For fresh databases, add it to 001_create_tables.sql instead

-- Users table (human users with username/password)
-- user.id IS the player_id (they are the same)
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,  -- Not AUTOINCREMENT - manually specify to match player
  username VARCHAR UNIQUE NOT NULL,
  password_hash VARCHAR NOT NULL,
  display_name VARCHAR NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(id) REFERENCES players(id)
);

-- Bots table (multiple bots per user, share player_id with user)
CREATE TABLE IF NOT EXISTS bots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,  -- user_id IS the player_id
  bot_name VARCHAR NOT NULL,  -- Internal identifier (e.g., "bot1", "my_ml_bot")
  bot_display_name VARCHAR NOT NULL,  -- User-facing name (e.g., "My ML Bot")
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(user_id, bot_name),
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
