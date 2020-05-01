use crate::group::stats::PrimaryStats;
use crate::group::Group;
use crate::team::TeamId;

/// FIFA World Cup 2018 Order
///
/// https://www.fifa.com/worldcup/news/tie-breakers-for-russia-2018-groups
///
///First step: Pursuant to the criteria listed in art. 32 (5) lit. a) to c) of the Competition Regulations.
///
///1. Greatest number of points obtained in all group matches
///2. Goal difference in all group matches
///3. Greatest number of goals scored in all group matches.
///
///Second step: If two or more teams are equal on the basis of the first step (see example in Table 1), their ranking will be determined by applying to the group matches between the teams concerned the criteria listed in art. 32 (5) lit. d) to h) in the order of their listing.
///
/// 4. Greatest number of points obtained in the group matches between the teams concerned
/// 5. Goal difference resulting from the group matches between the teams concerned
/// 6. Greater number of goals scored in all group matches between the teams concerned
/// 7. Greater number of points obtained in the fair play conduct of the teams based on yellow and red cards received in all group matches
///     - Yellow card: -1 points
///     - Indirect red card (second yellow card): -3 points
///     - Direct red card: -4 points
///     - Yellow card and direct red card: -5 points
/// 8. Drawing of lots by the FIFA.
///
/// TODO: Complete rules 4-8
pub fn fifa_2018_rules(group: &Group) -> GroupOrder {
    let mut team_stats: Vec<(TeamId, PrimaryStats)> = group
        .primary_stats()
        .into_iter()
        .map(|(team, stat)| (team, stat))
        .collect();
    team_stats.sort_by_key(|x| x.1);
    // Need to reverse the iter since the sort is ascending
    GroupOrder(team_stats.into_iter().rev().map(|(team, _)| team).collect())
}

#[derive(Clone, Copy, Debug)]
pub struct GroupRank(pub usize);

#[derive(Debug, PartialEq)]
pub struct GroupOrder(Vec<TeamId>);

impl GroupOrder {
    pub fn winner(&self) -> TeamId {
        self[GroupRank(0)]
    }
    pub fn runner_up(&self) -> TeamId {
        self[GroupRank(1)]
    }
    pub fn iter(&self) -> impl Iterator<Item = &TeamId> {
        self.0.iter()
    }
}

impl std::ops::Index<GroupRank> for GroupOrder {
    type Output = TeamId;
    fn index(&self, idx: GroupRank) -> &Self::Output {
        &self.0[idx.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::game::PlayedGroupGame;
    use crate::group::order;
    use crate::group::Group;
    use crate::Date;

    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn simple_point_order() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::dummy()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 0), (0, 0), Date::dummy()).unwrap();
        let group = Group::try_new(vec![game_1, game_2], vec![]).unwrap();
        let group_order = order::fifa_2018_rules(&group);
        let true_order = GroupOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// FairPlayScore necessary for strict order
    /// Is the sorting deterministic if the order is not strict?
    #[test]
    fn fair_play_order() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 0), (1, 4), Date::dummy()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (0, 0), (0, 2), Date::dummy()).unwrap();
        let group = Group::try_new(vec![game_1, game_2], vec![]).unwrap();
        let group_order = order::fifa_2018_rules(&group);
        let true_order = GroupOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// Two teams with same points, diff and score.
    /// The internal game decides
    #[test]
    fn internal_game() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (1, 0), (0, 0), Date::dummy()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 0, 2, (0, 1), (0, 0), Date::dummy()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 1, 2, (1, 0), (0, 0), Date::dummy()).unwrap();
        let group = Group::try_new(vec![game_1, game_2, game_3], vec![]).unwrap();
        let group_order = order::fifa_2018_rules(&group);
        let true_order = GroupOrder(vec![0, 1, 2].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }
}
