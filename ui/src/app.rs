use crate::data::{get_groups, get_teams};
use crate::game::ScoreInput;
use crate::group::view_group_play;
use seed::{prelude::*, *};
use wwc_core::{
    game::GameId,
    group::{
        order::{fifa_2018, Random, Rules},
        GroupId, Groups,
    },
    team::Teams,
};
const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";

struct Model {
    groups: Groups,
    teams: Teams,
    base_url: Url,
    group_rules: Rules<Random>,
}

pub(crate) enum Msg {
    UrlChanged(subs::UrlChanged),
    FetchTeams,
    TeamsFetched(fetch::Result<Teams>),
    FetchGroups,
    GroupsFetched(fetch::Result<Groups>),
    PlayGame(ScoreInput),
    UnplayGame(GroupId, GameId),
    UnfinishedScoreInput,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::FetchTeams });
    orders.perform_cmd(async { Msg::FetchGroups });
    Model {
        groups: Groups::new(),
        teams: Teams::new(),
        base_url: Url::new(),
        group_rules: fifa_2018(),
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchTeams => {
            log!("Fetching teams");
            orders
                .skip()
                .perform_cmd(async { Msg::TeamsFetched(get_teams().await) });
        }

        Msg::TeamsFetched(Ok(teams)) => {
            log!(&format!("Fetched {} teams", teams.len()));
            model.teams = teams;
        }

        Msg::TeamsFetched(Err(fetch_error)) => {
            error!("Error fetching teams {}", fetch_error);
        }

        Msg::FetchGroups => {
            log!("Fetching groups");
            orders
                .skip()
                .perform_cmd(async { Msg::GroupsFetched(get_groups().await) });
        }

        Msg::GroupsFetched(Ok(mut groups)) => {
            log!(&format!("Fetched {} groups", groups.len()));
            // Unplay a few games for display testing.
            let group_a = groups.get_mut(&GroupId::try_new('a').unwrap()).unwrap();
            let game_idcs: Vec<_> = group_a.played_games().map(|game| game.id).collect();
            game_idcs.iter().for_each(|idx| group_a.unplay_game(*idx));
            model.groups = groups;
        }

        Msg::GroupsFetched(Err(fetch_error)) => {
            error!("Error fetching groups {}", fetch_error);
        }

        Msg::PlayGame(input) => {
            let group = model.groups.get_mut(&input.group_id).unwrap();
            group.play_game(input.game_id, input.score);

        }
        Msg::UnplayGame(group_id, game_id) => {
            log!("Replaying game {} in group {}", game_id, group_id);
            let group = model.groups.get_mut(&group_id).unwrap();
            group.unplay_game(game_id);
        }
        _ => {}
    }
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_header(),
        view_group_play(&model.groups, &model.teams, &model.group_rules),
        // view_play_off(&model.playoff, &model.teams)
    ]
}

fn view_header() -> Node<Msg> {
    header![C!["header"], h1!["Wednesday world cup"],]
}

#[wasm_bindgen(start)]
pub fn start() {
    let root_element = document()
        .get_elements_by_class_name("tournament")
        .item(0)
        .expect("element with the class `tournament`");

    App::start(root_element, init, update, view);
}
