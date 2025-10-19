use crate::data::{clear_preds, get_groups_played_with_preds, get_teams, save_preds};
use crate::game::ScoreInput;
use crate::group::view_group_play;
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;
use wwc_core::{
    game::GameId,
    group::{GroupId, Groups},
    player::{Player, PlayerPredictions, Prediction},
    team::Teams,
};

#[component]
pub fn App() -> impl IntoView {
    // Reactive signals for model state
    let groups = RwSignal::new(Groups::new());
    let teams = RwSignal::new(Teams::new());
    let player = RwSignal::new(Player::dummy());

    // Fetch teams on mount
    Effect::new(move |_| {
        spawn_local(async move {
            console::log_1(&"Fetching teams".into());
            match get_teams().await {
                Ok(fetched_teams) => {
                    console::log_1(&format!("Fetched {} teams", fetched_teams.len()).into());
                    teams.set(fetched_teams);
                }
                Err(e) => {
                    console::error_1(&format!("Error fetching teams: {}", e).into());
                }
            }
        });
    });

    // Fetch groups on mount
    Effect::new(move |_| {
        spawn_local(async move {
            console::log_1(&"Fetching groups".into());
            let player_id = player.get_untracked().id();
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
                console::log_1(&format!("Game played successfully").into());
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
        let current_player = player.get();
        spawn_local(async move {
            console::log_1(&"Saving preds".into());
            let player_preds = PlayerPredictions::new(
                current_player.id(),
                current_groups
                    .iter()
                    .flat_map(|(_, group)| group.played_games())
                    .map(|game| Prediction::from(*game))
                    .collect(),
            );
            match save_preds(player_preds).await {
                Ok(_) => {
                    console::log_1(&"Preds saved successfully".into());
                }
                Err(e) => {
                    console::error_1(&format!("Error saving preds: {}", e).into());
                }
            }
        });
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
        spawn_local(async move {
            match clear_preds().await {
                Ok(_) => {
                    console::log_1(&"Preds cleared".into());
                }
                Err(e) => {
                    console::error_1(&format!("Error clearing preds: {}", e).into());
                }
            }
        });
    };

    view! {
        <div>
            <header class="header">
                <h1>"Wednesday world cup"</h1>
            </header>
            <button on:click=save_preds_action>"Save preds"</button>
            <br/>
            <button on:click=clear_preds_action>"Clear preds"</button>
            {move || view_group_play(
                groups.get(),
                teams.get(),
                play_game,
                unplay_game
            )}
        </div>
    }
}
