use crate::app::Msg;
use crate::format::Format;
use crate::team::format_team_flag;
use seed::{prelude::*, *};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{PlayedGroupGame, Score, UnplayedGroupGame};
use wwc_core::group::GroupId;
use wwc_core::team::Teams;

impl<'a> Format<'a> for PlayedGroupGame {
    type Context = (&'a Teams, GroupId);
    fn format(&self, cxt: &(&Teams, GroupId)) -> Node<Msg> {
        let (teams, group_id) = cxt;
        let group_id = *group_id;
        let home_team = teams.get(&self.home).unwrap();
        let away_team = teams.get(&self.away).unwrap();
        let game_id = self.id;
        tr![
            C!["played_game"],
            el_key(&game_id),
            td![home_team.fifa_code.to_string()],
            td![format_team_flag(home_team)],
            td![self.score.home.to_string()],
            td![self.score.away.to_string()],
            td![away_team.fifa_code.to_string()],
            td![format_team_flag(away_team)],
            // button!["&#8635;", ev(Ev::Click, |_| Msg::ReplayGame),],
            button![
                "\u{1F504}",
                ev(Ev::Click, move |_| Msg::UnplayGame(group_id, game_id)),
            ],
        ]
    }
}

impl<'a> Format<'a> for UnplayedGroupGame {
    type Context = (&'a Teams, GroupId);
    fn format(&self, cxt: &(&Teams, GroupId)) -> Node<Msg> {
        let (teams, group_id) = cxt;
        let home_team = teams.get(&self.home).unwrap();
        let away_team = teams.get(&self.away).unwrap();
        let score_input = ScoreInput::placeholder(*group_id, self.id);
        tr![
            C!["played_game"],
            el_key(&self.id),
            td![home_team.fifa_code.to_string()],
            td![format_team_flag(home_team)],
            td![input![
                C!["game-score-input"],
                attrs![At::Size => 2],
                input_ev(Ev::Input, |score| {
                    if let Ok(score) = score_input.try_parse_score(score) {
                        Msg::PlayGame(score)
                    } else {
                        Msg::UnfinishedScoreInput
                    }
                })
            ]],
            td![""],
            td![away_team.fifa_code.to_string()],
            td![format_team_flag(away_team)],
        ]
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScoreInput {
    pub(crate) score: Score,
    pub(crate) group_id: GroupId,
    pub(crate) game_id: GameId,
}

impl ScoreInput {
    fn placeholder(group_id: GroupId, game_id: GameId) -> Self {
        let score = Score::from((GoalCount(0), GoalCount(0)));
        ScoreInput {
            score,
            group_id,
            game_id,
        }
    }

    fn try_parse_score(self, score: String) -> Result<Self, ()> {
        let score = score.parse().map_err(|_| ())?;
        Ok(ScoreInput { score, ..self })
    }
}
