CREATE TABLE games (
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
  played BOOLEAN NOT NULL DEFAULT 'f'
)
