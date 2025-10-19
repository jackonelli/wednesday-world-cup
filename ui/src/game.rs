use crate::team::format_team_flag;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wwc_core::game::GameId;
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
            <td>{game.score.home.to_string()}</td>
            <td>{game.score.away.to_string()}</td>
            <td>{away_team.fifa_code.to_string()}</td>
            <td><span class={away_flag}></span></td>
            <td>
                <button on:click=move |_| on_unplay(group_id, game_id)>
                    "\u{1F504}"
                </button>
            </td>
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

    let on_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            let target = ev.target().unwrap();
            let input: web_sys::HtmlInputElement = target.dyn_into().unwrap();
            let value = input.value();
            if let Ok(score) = value.parse::<GroupGameScore>() {
                on_play(ScoreInput::new(score, group_id, game_id));
                input.set_value(""); // Clear the input after successful play
            }
        }
    };

    let home_flag = format_team_flag(&home_team);
    let away_flag = format_team_flag(&away_team);

    view! {
        <tr class="played_game">
            <td>{home_team.fifa_code.to_string()}</td>
            <td><span class={home_flag}></span></td>
            <td>
                <input
                    class="game-score-input"
                    size=2
                    on:keydown=on_keydown
                />
            </td>
            <td>""</td>
            <td>{away_team.fifa_code.to_string()}</td>
            <td><span class={away_flag}></span></td>
        </tr>
    }
}
