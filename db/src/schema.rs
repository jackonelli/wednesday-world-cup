table! {
    games (id) {
        id -> Integer,
        type_ -> Text,
        home_team -> Integer,
        away_team -> Integer,
        home_result -> Nullable<Integer>,
        away_result -> Nullable<Integer>,
        home_penalty -> Nullable<Integer>,
        away_penalty -> Nullable<Integer>,
        home_fair_play -> Nullable<Integer>,
        away_fair_play -> Nullable<Integer>,
        played -> Bool,
    }
}

table! {
    group_game_map (game_id) {
        game_id -> Integer,
        group_id_ -> Text,
    }
}

table! {
    teams (id) {
        id -> Integer,
        name -> Text,
        fifa_code -> Text,
        iso2 -> Text,
        rank_ -> Integer,
    }
}

joinable!(group_game_map -> games (game_id));

allow_tables_to_appear_in_same_query!(games, group_game_map, teams,);
