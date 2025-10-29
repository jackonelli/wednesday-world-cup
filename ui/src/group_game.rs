use crate::team::format_team_flag;
use leptos::ev;
use leptos::prelude::*;
use wwc_core::game::GameId;
use wwc_core::game::GoalCount;
use wwc_core::group::GroupId;
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::team::Teams;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScoreInput {
    pub(crate) score: GroupGameScore,
    pub(crate) group_id: GroupId,
    pub(crate) game_id: GameId,
}

impl ScoreInput {
    pub fn new(score: GroupGameScore, group_id: GroupId, game_id: GameId) -> Self {
        ScoreInput {
            score,
            group_id,
            game_id,
        }
    }
}

#[component]
pub fn PlayedGameView(
    game: PlayedGroupGame,
    teams: Teams,
    group_id: GroupId,
    on_unplay: impl Fn(GroupId, GameId) + 'static,
) -> impl IntoView {
    let home_team = teams.get(&game.home).unwrap().clone();
    let away_team = teams.get(&game.away).unwrap().clone();
    let game_id = game.id;

    let home_flag = format_team_flag(&home_team);
    let away_flag = format_team_flag(&away_team);

    view! {
        <tr class="played_game">
            <td>{home_team.fifa_code.to_string()}</td>
            <td><span class={home_flag}></span></td>
            <td class="score-display">
                {game.score.home.to_string()}
                <span class="score-separator">"-"</span>
                {game.score.away.to_string()}
                <button class="unplay-button" on:click=move |_| on_unplay(group_id, game_id)>
                    "\u{1F504}"
                </button>
            </td>
            <td>{away_team.fifa_code.to_string()}</td>
            <td><span class={away_flag}></span></td>
        </tr>
    }
}

#[component]
pub fn UnplayedGameView(
    game: UnplayedGroupGame,
    teams: Teams,
    group_id: GroupId,
    on_play: impl Fn(ScoreInput) + 'static,
) -> impl IntoView {
    let home_team = teams.get(&game.home).unwrap().clone();
    let away_team = teams.get(&game.away).unwrap().clone();
    let game_id = game.id;

    let home_input_ref = NodeRef::<leptos::html::Input>::new();
    let away_input_ref = NodeRef::<leptos::html::Input>::new();

    let try_submit = move || {
        if let (Some(home_input), Some(away_input)) = (home_input_ref.get(), away_input_ref.get()) {
            let home_val = home_input.value();
            let away_val = away_input.value();

            if let (Ok(home), Ok(away)) = (home_val.parse::<u32>(), away_val.parse::<u32>()) {
                let score_str = format!("{}-{}", home, away);
                if let (Ok(home), Ok(away)) = (GoalCount::try_from(home), GoalCount::try_from(away))
                {
                    let score = GroupGameScore::new(home, away);
                    on_play(ScoreInput::new(score, group_id, game_id));
                    home_input.set_value("");
                    away_input.set_value("");
                }
            }
        }
    };

    let on_home_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Tab" || ev.key() == "Enter" {
            if let Some(away_input) = away_input_ref.get() {
                let _ = away_input.focus();
                ev.prevent_default();
            }
        }
    };

    let on_away_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            try_submit();
        }
    };

    let home_flag = format_team_flag(&home_team);
    let away_flag = format_team_flag(&away_team);

    view! {
        <tr class="played_game">
            <td>{home_team.fifa_code.to_string()}</td>
            <td><span class={home_flag}></span></td>
            <td class="score-input-container">
                <input
                    node_ref=home_input_ref
                    class="game-score-input"
                    type="number"
                    min="0"
                    size=1
                    on:keydown=on_home_keydown
                />
                <span class="score-separator">"-"</span>
                <input
                    node_ref=away_input_ref
                    class="game-score-input"
                    type="number"
                    min="0"
                    size=1
                    on:keydown=on_away_keydown
                />
            </td>
            <td>{away_team.fifa_code.to_string()}</td>
            <td><span class={away_flag}></span></td>
        </tr>
    }
}
