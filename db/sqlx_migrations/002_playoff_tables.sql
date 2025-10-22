-- Playoff games table (just the IDs)
-- Playoff games don't have known teams until BracketStructure resolves them
CREATE TABLE IF NOT EXISTS playoff_games (
  id INTEGER PRIMARY KEY NOT NULL
);

-- Playoff team sources (for building BracketStructure)
-- This defines HOW teams enter each playoff game
CREATE TABLE IF NOT EXISTS playoff_team_sources (
  game_id INTEGER PRIMARY KEY NOT NULL,

  -- Home team source
  home_source_type VARCHAR NOT NULL,  -- 'group_outcome' | 'winner_of' | 'loser_of'
  home_group_id CHAR,                 -- if group_outcome: 'A', 'B', etc.
  home_outcome VARCHAR,               -- if group_outcome: 'winner' | 'runner_up' | 'third_place'
  home_third_place_groups VARCHAR,    -- if third_place: JSON array like '["A","B","C"]'
  home_source_game_id INTEGER,        -- if winner_of/loser_of: source game id

  -- Away team source
  away_source_type VARCHAR NOT NULL,
  away_group_id CHAR,
  away_outcome VARCHAR,
  away_third_place_groups VARCHAR,
  away_source_game_id INTEGER,

  FOREIGN KEY(game_id) REFERENCES playoff_games(id)
);

-- Playoff game results (when played)
CREATE TABLE IF NOT EXISTS playoff_results (
  game_id INTEGER PRIMARY KEY NOT NULL,
  home_team INTEGER NOT NULL,
  away_team INTEGER NOT NULL,
  home_result INTEGER NOT NULL,
  away_result INTEGER NOT NULL,
  home_penalty INTEGER,
  away_penalty INTEGER,
  FOREIGN KEY(game_id) REFERENCES playoff_games(id),
  FOREIGN KEY(home_team) REFERENCES teams(id),
  FOREIGN KEY(away_team) REFERENCES teams(id)
);
