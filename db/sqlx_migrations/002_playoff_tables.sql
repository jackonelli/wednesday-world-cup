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

  FOREIGN KEY(game_id) REFERENCES games(id)
);

-- Playoff game results are stored in the existing games table
-- BracketState is built by filtering games where id IN (SELECT game_id FROM playoff_team_sources)
