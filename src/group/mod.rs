use crate::game::group::{PlayedGroupGame, PreGroupGame};
use crate::game::Game;
use crate::group::stats::{GroupStats, GroupTeamStats};
use crate::team::TeamId;
use std::collections::HashSet;
use thiserror::Error;
pub mod stats;

pub struct Group {
    teams: HashSet<TeamId>,
    upcoming_games: Vec<PreGroupGame>,
    played_games: Vec<PlayedGroupGame>,
}

impl Group {
    pub fn try_new(
        teams: HashSet<TeamId>,
        upcoming_games: Vec<PreGroupGame>,
        played_games: Vec<PlayedGroupGame>,
    ) -> Result<Self, GroupError> {
        check_game_teams_in_teams(&teams, &upcoming_games)?;
        check_game_teams_in_teams(&teams, &played_games)?;
        Ok(Self {
            teams,
            upcoming_games,
            played_games,
        })
    }

    fn init_group_stats(&self) -> GroupStats {
        self.teams
            .iter()
            .fold(GroupStats::default(), |mut acc, id| {
                acc.0.insert(*id, GroupTeamStats::default());
                acc
            })
    }

    fn rank_teams(&self) -> Vec<TeamId> {
        let stats = self.stats();
        todo!();
    }

    pub fn stats(&self) -> GroupStats {
        self.played_games
            .iter()
            .fold(self.init_group_stats(), |mut acc, game| {
                let (delta_home_stats, delta_away_stats) = game.stats();

                let stats = acc
                    .0
                    .get_mut(&game.home)
                    .expect("TeamId will always be present");
                *stats += delta_home_stats;

                let stats = acc
                    .0
                    .get_mut(&game.away)
                    .expect("TeamId will always be present");
                *stats += delta_away_stats;
                acc
            })
    }
}

fn check_game_teams_in_teams<T>(teams: &HashSet<TeamId>, games: &[T]) -> Result<(), GroupError>
where
    T: Game,
{
    let teams_in_games = games.iter().fold(HashSet::new(), |mut acc, game| {
        acc.insert(game.home_team());
        acc.insert(game.away_team());
        acc
    });
    if teams_in_games.is_subset(&teams) {
        Ok(())
    } else {
        Err(GroupError::GameTeamsNotInTeams)
    }
}

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Teams in group not unique")]
    GameTeamsNotInTeams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fair_play::FairPlay;
    use crate::game::group::{GroupGameId, PreGroupGame, Score};
    use crate::team::TeamId;
    use crate::Date;
    use std::collections::HashSet;
    #[test]
    fn stats_from_played_games() {
        let dummy_fair_play = FairPlay::default();
        let teams = [TeamId(0), TeamId(1), TeamId(2), TeamId(3)].iter().fold(
            HashSet::new(),
            |mut teams, team| {
                teams.insert(*team);
                teams
            },
        );
        let score = Score::new(2, 1);
        let game_1 = PreGroupGame::new(GroupGameId(1), TeamId(0), TeamId(1), Date {});
        let game_1 = game_1.play(score, dummy_fair_play.clone());

        let score = Score::new(0, 0);
        let game_2 = PreGroupGame::new(GroupGameId(2), TeamId(2), TeamId(3), Date {});
        let game_2 = game_2.play(score, dummy_fair_play.clone());

        let score = Score::new(2, 1);
        let game_3 = PreGroupGame::new(GroupGameId(3), TeamId(3), TeamId(1), Date {});
        let game_3 = game_3.play(score, dummy_fair_play.clone());

        let games = vec![game_1, game_2, game_3];
        let group = Group::try_new(teams, Vec::new(), games).unwrap();
        let stats = group.stats();
        let is_1 = stats.get(&TeamId(1)).unwrap();
        let is_2 = stats.get(&TeamId(2)).unwrap();
        let is_3 = stats.get(&TeamId(3)).unwrap();

        let true_1 = &GroupTeamStats::new(0, 2, 2, 4, 0);
        let true_2 = &GroupTeamStats::new(1, 1, 0, 0, 0);
        let true_3 = &GroupTeamStats::new(4, 2, 2, 1, 0);
        assert_eq!(is_1, true_1);
        assert_eq!(is_2, true_2);
        assert_eq!(is_3, true_3);
    }
}
