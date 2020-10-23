CREATE TABLE group_game_map (
  id INTEGER PRIMARY KEY NOT NULL,
  group_id_ CHAR NOT NULL,
  FOREIGN KEY(game_id) REFERENCES games(id)
)
