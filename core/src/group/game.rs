//! # Group game
//!
//! Data and functionality related to games in the group stage of the tournament.
//! This group is less restricted than a [`crate::playoff::game::PlayoffGame`] since the [`GroupGameScore`] is freer,
//! e.g. draws are allowed and there are no additional penalty shoot-out score.
//!
//! The two game structs [`UnplayedGroupGame`] and [`PlayedGroupGame`]
//! are the fundamental datastructure for the group; all other properties and statistics are
//! derived from them.
use crate::Date;
use crate::fair_play::FairPlayScore;
use crate::game::{Game, GameId, GoalCount, GoalDiff};
use crate::group::stats::GameStat;
use crate::group::{GroupError, GroupPoint};
use crate::team::TeamId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Unplayed group game
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct UnplayedGroupGame {
    pub id: GameId,
    pub home: TeamId,
    pub away: TeamId,
    date: Date,
}

impl UnplayedGroupGame {
    /// Fallible constructor.
    ///
    /// # Errors
    ///
    /// Enforces distinct team id's
    pub fn try_new<G: Into<GameId>, T: Into<TeamId>>(
        id: G,
        home: T,
        away: T,
        date: Date,
    ) -> Result<Self, GroupError> {
        let home = home.into();
        let away = away.into();
        let id = id.into();
        if home != away {
            Ok(Self {
                id,
                home,
                away,
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }

    /// Transform unplayed game to played.
    ///
    /// Only (public) way of constructing a [`PlayedGroupGame`].
    pub fn play(self, score: GroupGameScore, fair_play: FairPlayScore) -> PlayedGroupGame {
        PlayedGroupGame {
            id: self.id,
            home: self.home,
            away: self.away,
            date: self.date,
            score,
            fair_play,
        }
    }
}

impl Game for UnplayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

/// Played group game
///
/// Can only be constructed by invoking the [`UnplayedGroupGame::play`] method.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PlayedGroupGame {
    pub id: GameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: GroupGameScore,
    pub(crate) fair_play: FairPlayScore,
    pub(crate) date: Date,
}

impl PlayedGroupGame {
    /// Fallible constructor for a played group game
    ///
    /// Internal use only.
    ///
    /// # Errors
    ///
    /// Enforces distinct team id's
    #[cfg(test)]
    pub(crate) fn try_new<
        G: Into<GameId>,
        T: Into<TeamId>,
        S: Into<GroupGameScore>,
        F: Into<FairPlayScore>,
    >(
        id: G,
        home: T,
        away: T,
        score: S,
        fair_play: F,
        date: Date,
    ) -> Result<Self, GroupError> {
        let home = home.into();
        let away = away.into();
        if home != away {
            Ok(Self {
                id: id.into(),
                home,
                away,
                score: score.into(),
                fair_play: fair_play.into(),
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }

    /// Transform played game to unplayed.
    pub fn unplay(self) -> UnplayedGroupGame {
        UnplayedGroupGame {
            id: self.id,
            home: self.home,
            away: self.away,
            date: self.date,
        }
    }

    /// Points awarded to (home, away) teams respectively.
    pub fn points(&self) -> (GroupPoint, GroupPoint) {
        GroupPoint::stat(self)
    }

    /// Goal difference for (home, away) teams respectively.
    pub fn goal_diff(&self) -> (GoalDiff, GoalDiff) {
        GoalDiff::stat(self)
    }

    /// Goals scored for (home, away) teams respectively.
    pub fn goals_scored(&self) -> (GoalCount, GoalCount) {
        GoalCount::stat(self)
    }
}

impl Game for PlayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

/// Group game score.
///
/// Accepts any pair of non-negative integers
/// Score used for [`crate::group::game::PlayedGroupGame`]
///
/// Determines the outcome of a game which can be, win, loss or draw.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub struct GroupGameScore {
    pub home: GoalCount,
    pub away: GoalCount,
}

impl GroupGameScore {
    pub fn new<T: Into<GoalCount>>(home_goals: T, away_goals: T) -> Self {
        GroupGameScore {
            home: home_goals.into(),
            away: away_goals.into(),
        }
    }
    pub fn home_outcome(&self) -> GroupGameOutcome {
        match self.home.cmp(&self.away) {
            Ordering::Greater => GroupGameOutcome::Win,
            Ordering::Less => GroupGameOutcome::Lose,
            Ordering::Equal => GroupGameOutcome::Draw,
        }
    }
    pub fn away_outcome(&self) -> GroupGameOutcome {
        match self.home_outcome() {
            GroupGameOutcome::Win => GroupGameOutcome::Lose,
            GroupGameOutcome::Draw => GroupGameOutcome::Draw,
            GroupGameOutcome::Lose => GroupGameOutcome::Win,
        }
    }
}

impl std::fmt::Display for GroupGameScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.home, self.away)
    }
}

impl<T: Into<GoalCount>> From<(T, T)> for GroupGameScore {
    fn from(x: (T, T)) -> Self {
        let (home, away) = x;
        Self {
            home: home.into(),
            away: away.into(),
        }
    }
}

// TODO test.
impl FromStr for GroupGameScore {
    type Err = GroupError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let score_split: Vec<&str> = s.split('-').collect();
        let (home, away) = if score_split.len() != 2 {
            return Err(GroupError::GameScoreParse(String::from(s)));
        } else {
            (score_split[0], score_split[1])
        };
        //TODO: Better error handling
        let home = home
            .parse::<u32>()
            .map_err(|_err| GroupError::GameScoreParse(String::from(s)))?;
        let away = away
            .parse::<u32>()
            .map_err(|_err| GroupError::GameScoreParse(String::from(s)))?;
        let home_goals = GoalCount::try_from(home)
            .map_err(|_| GroupError::GameScoreParse(format!("{} (home goals too high)", s)))?;
        let away_goals = GoalCount::try_from(away)
            .map_err(|_| GroupError::GameScoreParse(format!("{} (away goals too high)", s)))?;
        Ok(GroupGameScore::from((home_goals, away_goals)))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub enum GroupGameOutcome {
    Win,
    Draw,
    Lose,
}

#[cfg(test)]
mod group_game {
    use super::*;
    #[test]
    fn home_win() {
        let game = PlayedGroupGame::try_new(
            0,
            0,
            1,
            (
                GoalCount::try_from(3).unwrap(),
                GoalCount::try_from(0).unwrap(),
            ),
            FairPlayScore::default(),
            Date::mock(),
        )
        .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn away_win() {
        let game = PlayedGroupGame::try_new(
            0,
            0,
            1,
            (
                GoalCount::try_from(0).unwrap(),
                GoalCount::try_from(2).unwrap(),
            ),
            FairPlayScore::default(),
            Date::mock(),
        )
        .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn draw() {
        let game = PlayedGroupGame::try_new(
            0,
            0,
            1,
            (
                GoalCount::try_from(0).unwrap(),
                GoalCount::try_from(0).unwrap(),
            ),
            FairPlayScore::default(),
            Date::mock(),
        )
        .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}

#[cfg(test)]
mod score {
    use super::*;
    #[test]
    fn correct_single_digits() {
        let true_score = GroupGameScore::new(
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(2).unwrap(),
        );
        let parsed_score = GroupGameScore::from_str("1-2").unwrap();
        assert_eq!(true_score, parsed_score);
    }

    #[test]
    fn correct_double_digits() {
        let true_score = GroupGameScore::new(
            GoalCount::try_from(11).unwrap(),
            GoalCount::try_from(22).unwrap(),
        );
        let parsed_score = GroupGameScore::from_str("11-22").unwrap();
        assert_eq!(true_score, parsed_score);
    }

    #[test]
    fn fails_gibberish() {
        let parsed_score = GroupGameScore::from_str("asödkfaäe");
        assert!(parsed_score.is_err());
    }
}
