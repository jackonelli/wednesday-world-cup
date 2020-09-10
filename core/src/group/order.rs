//! Group ordering
use crate::fair_play::FairPlayValue;
use crate::game::{GoalCount, GoalDiff};
use crate::group::stats::UnaryStat;
use crate::group::{Group, GroupError, GroupPoint};
use crate::team::{Rank, TeamId};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;

/// Fifa World Cup 2018 Order
///
/// https://www.fifa.com/worldcup/news/tie-breakers-for-russia-2018-groups
///
///First step: Pursuant to the criteria listed in art. 32 (5) lit. a) to c) of the Competition Regulations.
///
/// 1. Greatest number of points obtained in all group matches
/// 2. Goal difference in all group matches
/// 3. Greatest number of goals scored in all group matches.
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
/// TODO: Complete rule 8.
pub fn fifa_2018() -> Rules<Random> {
    let group_point: AllGroupStat<GroupPoint> = AllGroupStat::new();
    let goal_diff: AllGroupStat<GoalDiff> = AllGroupStat::new();
    let goal_count: AllGroupStat<GoalCount> = AllGroupStat::new();
    let int_group_point: InternalGroupStat<GroupPoint> = InternalGroupStat::new();
    let int_goal_diff: InternalGroupStat<GoalDiff> = InternalGroupStat::new();
    let int_goal_count: InternalGroupStat<GoalCount> = InternalGroupStat::new();
    let fair_play: InternalGroupStat<FairPlayValue> = InternalGroupStat::new();
    Rules {
        non_strict: vec![
            Box::new(group_point),
            Box::new(goal_diff),
            Box::new(goal_count),
            Box::new(int_group_point),
            Box::new(int_goal_diff),
            Box::new(int_goal_count),
            Box::new(fair_play),
        ],
        tiebreaker: Random {},
    }
}

/// Group ordering rules
///
/// All ordering rules should have an ordered list (vector) of subrules.
/// These subrules may define a non-strict ordering,
/// therefore a proper ordering rule must also define a tiebreaker which maps a (possibly)
/// non-strict ordering to a strict one.
///
/// E.g
///
/// - Fifa 2018 rules are an ordered list of 1-7 non-strict rules
/// and then random choice as the tiebreaker.
/// - Euro 2020 rules use a similar (but not the same) list of non-strict rules
/// but instead lets the team rank define the tiebreaker.
pub struct Rules<T: Tiebreaker> {
    non_strict: Vec<Box<dyn SubOrdering>>,
    tiebreaker: T,
}

/// Order group based on rules
///
/// First orders by a list of non-strict sub-orders.
/// If the sub-order is not strict, the rules' tiebreaker is used.
/// Does not panic since the unwrapping match arm is checked to be strict.
pub fn order_group<T: Tiebreaker>(group: &Group, rules: &Rules<T>) -> GroupOrder {
    let possibly_non_strict =
        non_strict_ordering(group, &rules.non_strict, NonStrictGroupOrder::init(group));
    if !possibly_non_strict.is_strict() {
        rules.tiebreaker.order(group, possibly_non_strict)
    } else {
        possibly_non_strict.try_into().unwrap()
    }
}

/// Try ordering a NonStrictGroupOrder
///
/// Returns the input group order if it is strict or if there are no more rules left to apply.
/// Otherwise recursively calls itself with the next rule.
fn non_strict_ordering(
    group: &Group,
    rules: &[Box<dyn SubOrdering>],
    sub_order: NonStrictGroupOrder,
) -> NonStrictGroupOrder {
    if sub_order.is_strict() || rules.len() < 1 {
        sub_order
    } else {
        let (current_rule, remaining_rules) = rules.split_at(1);
        let sub_order = sub_order
            .into_iter()
            .fold(NonStrictGroupOrder::empty(), |acc, x| {
                let new_order = current_rule[0].order(group, x);
                acc.extend(new_order)
            });
        non_strict_ordering(group, remaining_rules, sub_order)
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct GroupRank(pub usize);

/// List of TeamId's
///
/// Sorted from best to worst team.
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

impl IntoIterator for GroupOrder {
    type Item = TeamId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
        if value.is_strict() {
            Ok(GroupOrder(value.0.into_iter().map(|x| x[0]).collect()))
        } else {
            Err(GroupError::NonStrictOrder)
        }
    }
}

/// Intermediate group order representation
///
/// A non-strict group order is represented as a sorted vector of vectors of equal teams.
#[derive(Debug, PartialEq)]
pub struct NonStrictGroupOrder(Vec<Vec<TeamId>>);

impl NonStrictGroupOrder {
    fn empty() -> Self {
        NonStrictGroupOrder(vec![])
    }

    // TODO: Did not manage to impl w/ Iterator trait.
    fn iter(&self) -> impl Iterator<Item = &Vec<TeamId>> {
        self.0.iter()
    }

    /// Initialise an equal order
    ///
    /// A group with all teams equal are represented as a vector with a single element,
    /// where this element is a vector containing all the teams in the group.
    fn init(group: &Group) -> Self {
        NonStrictGroupOrder(vec![group.teams().collect()])
    }

    /// Strict ordering check
    ///
    /// Check if all subgroups (with equal elements) are of size 1.
    /// Subgroup s with |s| > 1 => non-strict ordering
    /// Subgroup s with |s| < 1 (= 0) => Bug, trivial subgroups are not removed correctly.
    fn is_strict(&self) -> bool {
        self.iter().all(|x| x.len() == 1)
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

impl IntoIterator for NonStrictGroupOrder {
    type Item = Vec<TeamId>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Total, but not strict order
///
/// A complete order of a group is defined by a prioritised list of orders
/// which implements this trait. I.e. they can take a vector of teams and split them into a
/// NonStrictGroupOrder.
pub trait SubOrdering {
    fn order(&self, group: &Group, order: Vec<TeamId>) -> NonStrictGroupOrder;
}

/// Ordering stat based on all games in the group
///
/// SubOrdering which orders by a metric based on a UnaryStat.
/// The metric is calculated from all games in the group, regardless of the subset of teams being
/// ordered.
///
/// AllGroupStat sub-orderings based on points, goal difference and goals scored are commonly the
/// first three sub-orderings in a group rule.
struct AllGroupStat<T: UnaryStat>(std::marker::PhantomData<T>);

impl<T: UnaryStat> AllGroupStat<T> {
    fn new() -> Self {
        AllGroupStat(std::marker::PhantomData::<T>)
    }
}

impl<T: UnaryStat + Ord + Copy> SubOrdering for AllGroupStat<T> {
    fn order(&self, group: &Group, order: Vec<TeamId>) -> NonStrictGroupOrder {
        // TODO: Not efficient to calc stats for all teams, but efficient is not very important
        // here.
        let stats_all_teams = T::team_stats(group);
        let mut team_stats: Vec<(TeamId, T)> = order
            .into_iter()
            .map(|id| (id, stats_all_teams.get(&id)))
            .filter(|(_, x)| x.is_some())
            .map(|(id, x)| (id, *x.unwrap()))
            .collect();
        team_stats.sort_by_key(|x| x.1);
        let team_stats = team_stats;
        let (_, new_order) = team_stats.iter().rev().fold(
            (team_stats[0].1, NonStrictGroupOrder::empty()),
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

/// Ordering stat based on the internal games in a teams subset
///
/// SubOrdering which orders by a metric based on a UnaryStat.
/// The metric is calculated from the games in the group, where both teams involved are members of
/// the subset of teams being ordered.
struct InternalGroupStat<T: UnaryStat>(std::marker::PhantomData<T>);

impl<T: UnaryStat> InternalGroupStat<T> {
    fn new() -> Self {
        InternalGroupStat(std::marker::PhantomData::<T>)
    }
}

impl<T: UnaryStat + Ord + Copy> SubOrdering for InternalGroupStat<T> {
    fn order(&self, group: &Group, order: Vec<TeamId>) -> NonStrictGroupOrder {
        let stats_all_teams = T::internal_team_stats(group, &HashSet::from_iter(&order));
        let mut team_stats: Vec<(TeamId, T)> = order
            .into_iter()
            .map(|id| (id, stats_all_teams.get(&id)))
            .filter(|(_, x)| x.is_some())
            .map(|(id, x)| (id, *x.unwrap()))
            .collect();
        team_stats.sort_by_key(|x| x.1);
        let team_stats = team_stats;
        let (_, new_order) = team_stats.iter().rev().fold(
            (team_stats[0].1, NonStrictGroupOrder::empty()),
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

/// Associated with [`Rules`](struct.Rules.html) to ensure strict total order.
pub trait Tiebreaker {
    fn order(&self, group: &Group, non_strict: NonStrictGroupOrder) -> GroupOrder {
        GroupOrder(non_strict.0.into_iter().fold(Vec::new(), |mut acc, x| {
            if x.len() == 1 {
                acc.push(x[0]);
                acc
            } else {
                [acc, self.order_sub_group(group, &x).0].concat()
            }
        }))
    }

    fn order_sub_group(&self, _: &Group, order: &[TeamId]) -> GroupOrder {
        //TODO: There must be a more efficient way to do this?
        let mut tmp_order = order.to_vec();
        tmp_order.sort_by(|a, b| self.cmp(*a, *b));
        GroupOrder(tmp_order.into_iter().map(|x| x).rev().collect())
    }

    /// Answers a comparison posed like this:
    /// "Compare id_1 to id_2". I.e. if the return value is `Ordering::Greater` it means that id_1
    /// is greater than id_2.
    fn cmp(&self, id_1: TeamId, id_2: TeamId) -> Ordering;
}

/// Manual tiebreaker
///
/// For actual tournaments some tiebreakers are out of our control,
/// e.g. the Fifa random tiebreaker where the lot is drawn externally,
/// This struct provides a manual tiebreaker in order to comply with actual events.
pub struct Manual(HashMap<(TeamId, TeamId), Ordering>);

impl Tiebreaker for Manual {
    fn cmp(&self, id_1: TeamId, id_2: TeamId) -> Ordering {
        *self
            .0
            .get(&(id_1, id_2))
            .expect("Comparison does not exist")
    }
}

/// Random tiebreaker
pub struct Random;

impl Tiebreaker for Random {
    fn cmp(&self, _id_1: TeamId, _id_2: TeamId) -> Ordering {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() > 0.5 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Rank tiebreaker
pub struct UefaRanking(HashMap<TeamId, Rank>);

impl UefaRanking {
    pub fn try_new(
        groups: &[Group],
        ranking_map: HashMap<TeamId, Rank>,
    ) -> Result<Self, GroupError> {
        // TODO: Why does this need to be mut?
        let mut all_teams = groups.iter().flat_map(|x| x.teams());
        let exists = all_teams.all(|x| ranking_map.get(&x).is_some());
        if exists {
            Ok(UefaRanking(ranking_map))
        } else {
            Err(GroupError::GenericError)
        }
    }
}

impl Tiebreaker for UefaRanking {
    /// Comparison by Uefa ranking
    ///
    /// Panics if the team id's are not in `self.ranking_map`
    /// Internally ok since the fallible constructor [`try_new`](struct.UefaRanking.html#method.try_new) ensures that the teams in the groups are a subset of the `ranking_map`.
    fn cmp(&self, id_1: TeamId, id_2: TeamId) -> Ordering {
        let rank_1 = self.0.get(&id_1).unwrap();
        let rank_2 = self.0.get(&id_2).unwrap();
        // Switch the order of comparison here so that a small rank is considered better than a
        // large one.
        rank_2.cmp(&rank_1)
    }
}

#[cfg(test)]
mod ordering_tests {
    use super::*;
    use crate::group::game::PlayedGroupGame;
    use crate::group::Group;
    use crate::Date;
    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn new_point_order() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 1), (0, 0), Date::mock()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 0, 3, (0, 1), (0, 0), Date::mock()).unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3]).unwrap();
        let rules = fifa_2018();
        let group_order = order_group(&group, &rules);
        let true_order = GroupOrder(vec![3, 1, 2, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// Different ordering based on points vs scored goals
    #[test]
    fn points_scored_goals_discrepancy() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 1), (0, 0), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 0, 2, (0, 0), (0, 0), Date::mock()).unwrap();
        let game_4 = PlayedGroupGame::try_new(3, 1, 3, (5, 5), (0, 0), Date::mock()).unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3, game_4]).unwrap();
        let rules = fifa_2018();
        let group_order = order_group(&group, &rules);
        let true_order = GroupOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn prim_stats_orders() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (1, 0), (0, 0), Date::mock()).unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2]).unwrap();
        let rules = fifa_2018();
        let group_order = order_group(&group, &rules);
        let true_order = GroupOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// FairPlayScore necessary for strict order.
    /// NB: The sorting is not deterministic if the order is not strict.
    #[test]
    fn fair_play_order() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (0, 0), (1, 4), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 2, 3, (0, 0), (0, 2), Date::mock()).unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2]).unwrap();
        let rules = fifa_2018();
        let group_order = order_group(&group, &rules);
        let true_order = GroupOrder(vec![2, 0, 3, 1].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// Two teams with same points, diff and score.
    /// The internal game decides
    #[test]
    fn internal_game() {
        let game_1 = PlayedGroupGame::try_new(0, 0, 2, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 1, 2, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 1, 2, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_4 = PlayedGroupGame::try_new(3, 0, 1, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_5 = PlayedGroupGame::try_new(4, 0, 3, (0, 1), (0, 0), Date::mock()).unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3, game_4, game_5]).unwrap();
        let rules = fifa_2018();
        let group_order = order_group(&group, &rules);
        let true_order = GroupOrder(vec![0, 1, 3, 2].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }
}

#[cfg(test)]
mod tiebreaker_test {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn uefa_rank() {
        let mut ranking = HashMap::new();
        ranking.insert(TeamId(0), Rank(1));
        ranking.insert(TeamId(1), Rank(2));
        let ranking = UefaRanking(ranking);
        assert_eq!(ranking.cmp(TeamId(0), TeamId(1)), Ordering::Greater);
    }
}
