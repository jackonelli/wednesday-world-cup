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
    groups (unik) {
        unik -> Text,
        id -> Text,
        game_id -> Integer,
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

joinable!(groups -> games (game_id));

allow_tables_to_appear_in_same_query!(
    games,
    groups,
    teams,
);
