use crate::group::stats::PrimaryStats;
use crate::group::{Group, GroupError, GroupPoint};
use crate::team::TeamId;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

/// Fifa World Cup 2018 Order
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
pub struct Fifa2018;

pub struct Rules<T: Tiebreaker> {
    non_strict: Vec<Box<dyn SubOrdering>>,
    tiebreaker: T,
}

pub fn order_group<T: Tiebreaker>(group: &Group, rules: Rules<T>) -> GroupOrder {
    let possibly_non_strict = ordering(group, &rules.non_strict, NonStrictGroupOrder::init(group));
    if !possibly_non_strict.is_strict() {
        rules.tiebreaker.order(group, possibly_non_strict)
    } else {
        possibly_non_strict.try_into().unwrap()
    }
}

fn ordering(
    group: &Group,
    rules: &[Box<dyn SubOrdering>],
    sub_order: NonStrictGroupOrder,
) -> NonStrictGroupOrder {
    println!("Sub order: {:?}", sub_order);
    if sub_order.is_strict() || rules.len() < 1 {
        sub_order
    } else {
        let (current_rule, remaining_rules) = rules.split_at(1);
        let sub_order = sub_order
            .0
            .into_iter()
            .fold(NonStrictGroupOrder::empty(), |acc, x| {
                let new_order = current_rule[0].order(group, x);
                acc.extend(new_order)
            });
        ordering(group, remaining_rules, sub_order)
    }
}

pub fn fifa_2018_rules(group: &Group) -> GroupOrder {
    let mut primary_stats: Vec<(TeamId, PrimaryStats)> = group
        .primary_stats()
        .into_iter()
        .map(|(team, stat)| (team, stat))
        .collect();
    primary_stats.sort_by_key(|x| x.1);
    // Need to reverse the iter since the sort is ascending
    GroupOrder(
        primary_stats
            .into_iter()
            .rev()
            .map(|(team, _)| team)
            .collect(),
    )
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

impl TryFrom<NonStrictGroupOrder> for GroupOrder {
    type Error = GroupError;

    fn try_from(value: NonStrictGroupOrder) -> Result<Self, Self::Error> {
        if !value.is_strict() {
            return Err(GroupError::NonStrictOrder);
        }
        Ok(GroupOrder(value.0.into_iter().map(|x| x[0]).collect()))
    }
}

pub trait Tiebreaker {
    fn order(&self, group: &Group, order: NonStrictGroupOrder) -> GroupOrder;
}

pub struct Random;

impl Tiebreaker for Random {
    fn order(&self, group: &Group, order: NonStrictGroupOrder) -> GroupOrder {
        todo!();
    }
}

pub trait SubOrdering {
    fn order(&self, group: &Group, order: Vec<TeamId>) -> NonStrictGroupOrder;
}

impl<T: UnaryStat> SubOrdering for T {
    fn order(&self, group: &Group, order: Vec<TeamId>) -> NonStrictGroupOrder {
        let all_points = T::stat(group);
        let mut point_stats: Vec<(TeamId, T::Stat)> = order
            .into_iter()
            .map(|id| (id, all_points.get(&id)))
            .filter(|(_, x)| x.is_some())
            .map(|(id, x)| (id, *x.unwrap()))
            .collect();
        point_stats.sort_by_key(|x| x.1);
        let point_stats = point_stats;
        let (_, new_order) = point_stats.iter().rev().fold(
            (point_stats[0].1, NonStrictGroupOrder::empty()),
            |acc, x| {
                if acc.0 == x.1 {
                    (x.1, acc.1.extend_sub_order(x.0))
                } else {
                    (x.1, acc.1.add_sub_order(x.0))
                }
            },
        );
        new_order
    }
}

pub trait UnaryStat {
    type Stat: Ord + Copy;
    fn stat(group: &Group) -> HashMap<TeamId, Self::Stat>;
}

pub struct PointOrder {}

impl UnaryStat for PointOrder {
    type Stat = GroupPoint;

    fn stat(group: &Group) -> HashMap<TeamId, Self::Stat> {
        group.points()
    }
}

#[derive(Debug, PartialEq)]
pub struct NonStrictGroupOrder(Vec<Vec<TeamId>>);

impl NonStrictGroupOrder {
    fn empty() -> Self {
        NonStrictGroupOrder(vec![])
    }

    fn init(group: &Group) -> Self {
        NonStrictGroupOrder(vec![group.teams().collect()])
    }

    /// Strict ordering check
    ///
    /// Check if all subgroups (with equal elements) are of size 1.
    /// Subgroup s with |s| > 1 => non-strict ordering
    /// Subgroup s with |s| < 1 (= 0) => Bug, trivial subgroups are not removed correctly.
    fn is_strict(&self) -> bool {
        self.0.iter().all(|x| x.len() == 1)
    }

    fn extend_sub_order(mut self, team: TeamId) -> Self {
        let mut new_inner = self.0.pop().unwrap_or(vec![]);
        new_inner.push(team);
        self.0.push(new_inner);
        self
    }

    fn add_sub_order(mut self, team: TeamId) -> Self {
        self.0.push(vec![team]);
        self
    }

    fn extend(self, sub_order: NonStrictGroupOrder) -> Self {
        NonStrictGroupOrder([&self.0[..], &sub_order.0[..]].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::game::PlayedGroupGame;
    use crate::group::Group;
    use crate::Date;
    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn new_point_order() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::dummy()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 1), (0, 0), Date::dummy()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 0, 3, (0, 1), (0, 0), Date::dummy()).unwrap();
        let group = Group::try_new(vec![game_1, game_2, game_3], vec![]).unwrap();
        let rules = Rules {
            non_strict: vec![Box::new(PointOrder {})],
            tiebreaker: Random {},
        };
        let group_order = order_group(&group, rules);
        let true_order = GroupOrder(vec![3, 1, 2, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn prim_stats_orders() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::dummy()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 0), (0, 0), Date::dummy()).unwrap();
        let group = Group::try_new(vec![game_1, game_2], vec![]).unwrap();
        let group_order = fifa_2018_rules(&group);
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
        let group_order = fifa_2018_rules(&group);
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
        let group_order = fifa_2018_rules(&group);
        let true_order = GroupOrder(vec![0, 1, 2].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }
}
