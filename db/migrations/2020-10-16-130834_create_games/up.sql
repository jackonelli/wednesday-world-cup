CREATE TABLE games (
  id INTEGER PRIMARY KEY NOT NULL,
  type_ VARCHAR NOT NULL,
  home_team INTEGER NOT NULL,
  --FOREIGN KEY(home_team) REFERENCES teams(id)
  away_team INTEGER NOT NULL,
  --FOREIGN KEY(away_team) REFERENCES teams(id)
  home_result INTEGER,
  away_result INTEGER,
  home_penalty INTEGER,
  away_penalty INTEGER,
  home_fair_play INTEGER,
  away_fair_play INTEGER,
  played BOOLEAN NOT NULL DEFAULT 'f'
)
