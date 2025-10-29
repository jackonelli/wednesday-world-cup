-- Create teams table
CREATE TABLE IF NOT EXISTS teams (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR NOT NULL,
  fifa_code VARCHAR NOT NULL,
  rank_ INTEGER NOT NULL
);

-- Create games table
CREATE TABLE IF NOT EXISTS games (
  id INTEGER PRIMARY KEY NOT NULL,
  type_ VARCHAR NOT NULL,
  home_team INTEGER NOT NULL,
  away_team INTEGER NOT NULL,
  home_result INTEGER,
  away_result INTEGER,
  home_penalty INTEGER,
  away_penalty INTEGER,
  home_fair_play INTEGER,
  away_fair_play INTEGER,
  played BOOLEAN NOT NULL DEFAULT 0
);

-- Create group_game_map table
CREATE TABLE IF NOT EXISTS group_game_map (
  id INTEGER PRIMARY KEY NOT NULL,
  group_id_ CHAR NOT NULL,
  FOREIGN KEY(id) REFERENCES games(id)
);

-- Create players table
CREATE TABLE IF NOT EXISTS players (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR NOT NULL
);

-- Create preds table
CREATE TABLE IF NOT EXISTS preds (
  id INTEGER PRIMARY KEY NOT NULL,
  player_id INTEGER NOT NULL,
  game_id INTEGER NOT NULL,
  home_result INTEGER NOT NULL,
  away_result INTEGER NOT NULL,
  bot_name VARCHAR,  -- NULL for human predictions, bot identifier for bot predictions
  FOREIGN KEY(game_id) REFERENCES games(id),
  FOREIGN KEY(player_id) REFERENCES players(id)
);
