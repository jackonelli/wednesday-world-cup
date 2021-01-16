use crate::format::Format;
use crate::format_team_flag;
use crate::Msg;
use seed::{prelude::*, *};
use wwc_core::game::GoalCount;
use wwc_core::group::game::{GroupGameId, PlayedGroupGame, Score, UnplayedGroupGame};
use wwc_core::group::GroupId;
use wwc_core::team::Teams;

impl Format<'_> for PlayedGroupGame {
    type Context = Teams;
    fn format(&self, cxt: &Teams) -> Node<Msg> {
        let home_team = cxt.get(&self.home).unwrap();
        let away_team = cxt.get(&self.away).unwrap();
        tr![
            C!["played_game"],
            el_key(&self.id),
            td![home_team.fifa_code.to_string()],
            td![format_team_flag(home_team)],
            td![self.score.home.to_string()],
            td![self.score.away.to_string()],
            td![away_team.fifa_code.to_string()],
            td![format_team_flag(away_team)],
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
                    Msg::PlayGame(score_input.update_score(score))
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
    pub(crate) game_id: GroupGameId,
}

impl ScoreInput {
    fn placeholder(group_id: GroupId, game_id: GroupGameId) -> Self {
        let score = Score::from((GoalCount(0), GoalCount(0)));
        ScoreInput {
            score,
            group_id,
            game_id,
        }
    }

    fn update_score(self, score: String) -> Self {
        let score = score.parse().unwrap();
        ScoreInput { score, ..self }
    }
}