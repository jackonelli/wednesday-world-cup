use crate::auth::{AuthState, logout};
use crate::data::{
    clear_my_preds, get_groups_played_with_preds, get_me, get_playoff_team_sources, get_teams,
    save_preds,
};
use crate::group::view_group_play;
use crate::group_game::ScoreInput;
use crate::playoff::PlayoffBracketView;
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;
use wwc_core::{
    game::GameId,
    group::{GroupId, Groups, order::fifa_2018_rules},
    player::{PlayerPredictions, Prediction},
    playoff::{BracketStructure, TeamSource},
    team::Teams,
};

#[component]
pub fn PredictionsView() -> impl IntoView {
    let auth_state = expect_context::<RwSignal<AuthState>>();
    let display_name = expect_context::<RwSignal<Option<String>>>();

    // Extract auth data from context
    let (auth_token, player_id) = match auth_state.get_untracked() {
        AuthState::Authenticated { token, player_id } => {
            (RwSignal::new(Some(token)), player_id.into())
        }
        _ => (RwSignal::new(None), wwc_core::player::PlayerId::from(1)), // Fallback, shouldn't happen due to ProtectedRoute
    };

    // Reactive signals for model state
    let groups = RwSignal::new(Groups::new());
    // Teams and team_sources are read-only, we only need to set them once on load,
    // therefore it is better for reactivity to have separate read/write accessors.
    let (teams, set_teams) = signal(Teams::new());
    let (team_sources, set_team_sources) = signal(Vec::<(GameId, (TeamSource, TeamSource))>::new());

    // Fetch user info and update display_name (run once on mount)
    Effect::new(move |_| {
        if let Some(token) = auth_token.get_untracked() {
            let token_clone = token.clone();
            spawn_local(async move {
                match get_me(&token_clone).await {
                    Ok((user_display_name, _bot_name)) => {
                        console::log_1(&format!("Fetched user info: {}", user_display_name).into());
                        // Update display name (separate signal, won't trigger auth reactivity)
                        display_name.set(Some(user_display_name));
                    }
                    Err(e) => {
                        console::error_1(&format!("Error fetching user info: {}", e).into());
                    }
                }
            });
        }
    });

    // Fetch teams on mount (run once)
    Effect::new(move |_| {
        spawn_local(async move {
            console::log_1(&"Fetching teams".into());
            match get_teams().await {
                Ok(fetched_teams) => {
                    console::log_1(&format!("Fetched {} teams", fetched_teams.len()).into());
                    set_teams.set(fetched_teams);
                }
                Err(e) => {
                    console::error_1(&format!("Error fetching teams: {}", e).into());
                }
            }
        });
    });

    // Fetch groups on mount (run once)
    Effect::new(move |_| {
        spawn_local(async move {
            console::log_1(&"Fetching groups".into());
            match get_groups_played_with_preds(player_id).await {
                Ok(fetched_groups) => {
                    console::log_1(&format!("Fetched {} groups", fetched_groups.len()).into());
                    groups.set(fetched_groups);
                }
                Err(e) => {
                    console::error_1(&format!("Error fetching groups: {}", e).into());
                }
            }
        });
    });

    // Fetch playoff team sources on mount (run once)
    Effect::new(move |_| {
        spawn_local(async move {
            console::log_1(&"Fetching playoff team sources".into());
            match get_playoff_team_sources().await {
                Ok(sources) => {
                    console::log_1(&format!("Fetched {} playoff games", sources.len()).into());
                    set_team_sources.set(sources);
                }
                Err(e) => {
                    console::error_1(&format!("Error fetching playoff: {}", e).into());
                }
            }
        });
    });

    // Action to play a game
    let play_game = move |input: ScoreInput| {
        console::log_1(
            &format!(
                "Playing game {} in group {} with score {:?}",
                input.game_id, input.group_id, input.score
            )
            .into(),
        );
        groups.update(|groups| {
            if let Some(group) = groups.get_mut(&input.group_id) {
                group.play_game(input.game_id, input.score);
                console::log_1(&"Game played successfully".into());
            }
        });
    };

    // Action to unplay a game
    let unplay_game = move |group_id: GroupId, game_id: GameId| {
        console::log_1(&format!("Replaying game {} in group {}", game_id, group_id).into());
        groups.update(|groups| {
            if let Some(group) = groups.get_mut(&group_id) {
                group.unplay_game(game_id);
            }
        });
    };

    // Action to save predictions
    let save_preds_action = move |_| {
        let current_groups = groups.get();
        if let Some(token) = auth_token.get() {
            spawn_local(async move {
                console::log_1(&"Saving preds".into());
                let player_preds = PlayerPredictions::new(
                    player_id,
                    current_groups
                        .iter()
                        .flat_map(|(_, group)| group.played_games())
                        .map(|game| Prediction::from(*game))
                        .collect(),
                );
                match save_preds(player_preds, &token).await {
                    Ok(_) => {
                        console::log_1(&"Preds saved successfully".into());
                    }
                    Err(e) => {
                        console::error_1(&format!("Error saving preds: {}", e).into());
                    }
                }
            });
        }
    };

    // Action to clear predictions
    let clear_preds_action = move |_| {
        console::log_1(&"Clearing preds".into());
        groups.update(|groups| {
            groups.iter_mut().for_each(|(_, group)| {
                let tmp = group.clone();
                tmp.played_games()
                    .for_each(|game| group.unplay_game(game.id));
            });
        });
        if let Some(token) = auth_token.get() {
            spawn_local(async move {
                match clear_my_preds(&token).await {
                    Ok(_) => {
                        console::log_1(&"Preds cleared successfully".into());
                    }
                    Err(e) => {
                        console::error_1(&format!("Error clearing preds: {}", e).into());
                    }
                }
            });
        }
    };

    // Logout action
    let on_logout = move |_| {
        logout(auth_state);
    };

    // Derive bracket structure reactively from team sources
    let bracket = move || {
        let sources = team_sources.get();
        if sources.is_empty() {
            None
        } else {
            BracketStructure::from_team_sources(&sources).ok()
        }
    };

    view! {
        <div>
            <header class="header">
                <h1>"Wednesday world cup"</h1>
                <div class="user-info">
                    {move || {
                        if let Some(name) = display_name.get() {
                            view! { <span>"Logged in as: " {name}</span> }.into_any()
                        } else {
                            view! { <span>"Loading user info..."</span> }.into_any()
                        }
                    }}
                    " "
                    <button on:click=on_logout>"Logout"</button>
                </div>
            </header>
            <button on:click=save_preds_action>"Save preds"</button>
            <br/>
            <button on:click=clear_preds_action>"Clear preds"</button>
            {move || view_group_play(groups.get(), teams.get(), play_game, unplay_game)}
            {move || {
                let current_groups = groups.get();
                let has_groups = !current_groups.is_empty();
                if has_groups {
                    bracket()
                        .map(|b| {
                            view! {
                                <PlayoffBracketView
                                    bracket=b
                                    groups=current_groups
                                    teams=teams.get()
                                    rules=fifa_2018_rules()
                                />
                            }
                        })
                } else {
                    None
                }
            }}
        </div>
    }
}
