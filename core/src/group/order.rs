//! # Group ordering
//!
//! A group is ordered by a list of sub-orders, followed by a final [`Tiebreaker`], see [`Rules`].
//! The ordering is greedy in the sense that the next sub-order is only applied if the ordering is non-strict.
//! If there are no more sub-orders to apply, a tiebreaker ensures a strict ordering.
//!
//! Initially the unordered group looks like this:
//!
//! 1. SWE, GER, BRA, ITA
//! 2.
//! 3.
//! 4.
//!
//! which is represented as a list of list ([`NonStrictGroupOrder`]):
//!
//! `[[SWE, GER, BRA, ITA], [], [], []]`
//!
//! A rule is applied, group points for instance. The teams' group points are computed and the
//! sub-lists are split. Let's say this is the order based on the group points:
//!
//! 1. SWE, GER
//! 2. BRA
//! 3. ITA
//! 4.
//! i.e.
//!
//! `[[SWE, GER] [BRA], [ITA]]`
//!
//! The order is not strict yet, since SWE and GER are tied, at which point the next rule is
//! applied (goal diff perhaps).
//!
//! The sub-ordering stops when
//! - We have achieved a strict order or,
//! - There are no more rules to apply. Then the [`Tiebreaker`] is applied (random chance or
//! ranking) which guarantees a strict order.
//!
//! A system of greedy, atomic sub-orders is flexible since new rules can easily be composed from them.
//! The [`SubOrdering`] trait is auto-implemented for every struct that
//! implements [`GameStat`] + [`Ord`] + [`Copy`], making the
//! composition of new rules straightforward, see ([`euro_2020`], [`fifa_2018`]).
//!
//! ### A note on performance
//!
//! Smaller sub-orders can also be grouped together. E.g., you could collect group points, goal diff.
//! and goals scored into one struct, implement [`GameStat`], [`Ord`] and [`Copy`] for it and use that as a
//! sub-order. This seems to be more efficient since you would avoid iterating over the played games
//! once for each stat, but is in reality not trivially so.
//! It would be better in the worst case scenario but if it is likely that teams are
//! separable by points alone, then it would be wasteful not to take advantage of the greedy
//! approach.
//! A benchmark would be certainly be interesting.
use crate::fair_play::{FifaFairPlayValue, UefaFairPlayValue};
use crate::game::{GoalCount, GoalDiff};
use crate::group::stats::{GameStat, NumWins};
use crate::group::{Group, GroupError, GroupPoint};
use crate::team::{TeamId, TeamRank};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use super::Groups;

/// Ordering rules
///
/// All ordering rules have an ordered list of subrules.
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
pub fn order_group<T: Tiebreaker>(group: &Group, rules: &Rules<T>) -> TeamOrder {
    let possibly_non_strict = non_strict_group_ordering(
        group,
        &rules.non_strict,
        NonStrictOrder::init_from_group(group),
    );
    if !possibly_non_strict.is_strict() {
        rules.tiebreaker.order_teams(possibly_non_strict)
    } else {
        // Does not panic since the unwrapping match arm is checked to be strict.
        possibly_non_strict.try_into().unwrap()
    }
}

/// Order general set of teams based on rules
///
/// First orders by a list of non-strict sub-orders.
/// If the sub-order is not strict, the rules' tiebreaker is used.
pub(crate) fn order_teams<T: Tiebreaker>(
    teams: &HashMap<TeamId, &Group>,
    rules: &Rules<T>,
) -> TeamOrder {
    let possibly_non_strict = non_strict_teams_ordering(
        teams,
        &rules.non_strict,
        NonStrictOrder::init_from_teams(teams.iter().map(|(id, _)| *id)),
    );
    if !possibly_non_strict.is_strict() {
        rules.tiebreaker.order_teams(possibly_non_strict)
    } else {
        // Does not panic since the unwrapping match arm is checked to be strict.
        possibly_non_strict.try_into().unwrap()
    }
}

/// Try ordering teams across groups
///
/// Returns the input group order if it is strict or if there are no more rules left to apply.
/// Otherwise recursively calls itself with the next rule.
fn non_strict_teams_ordering(
    teams: &HashMap<TeamId, &Group>,
    rules: &[Box<dyn SubOrdering>],
    sub_order: NonStrictOrder,
) -> NonStrictOrder {
    if sub_order.is_strict() || rules.is_empty() {
        sub_order
    } else {
        let (current_rule, remaining_rules) = rules.split_at(1);
        // current_rule is always a vec with a single element,
        let current_rule = &current_rule[0];
        let sub_order = sub_order
            .into_iter()
            .fold(NonStrictOrder::empty(), |acc, x| {
                // Don't apply rule if the sub-order is already strict,
                // i.e. if x consists of a single TeamId
                // TODO: benchmark, possible that the allocation in the else branch is more costly.
                let new_order = if x.len() > 1 {
                    current_rule.order_teams(teams, x)
                } else {
                    NonStrictOrder::single(x)
                };

                acc.extend(new_order)
            });
        non_strict_teams_ordering(teams, remaining_rules, sub_order)
    }
}

/// Try ordering a group
///
/// Returns the input group order if it is strict or if there are no more rules left to apply.
/// Otherwise recursively calls itself with the next rule.
fn non_strict_group_ordering(
    group: &Group,
    rules: &[Box<dyn SubOrdering>],
    sub_order: NonStrictOrder,
) -> NonStrictOrder {
    if sub_order.is_strict() || rules.is_empty() {
        sub_order
    } else {
        let (current_rule, remaining_rules) = rules.split_at(1);
        // current_rule is always a vec with a single element,
        let current_rule = &current_rule[0];
        let sub_order = sub_order
            .into_iter()
            .fold(NonStrictOrder::empty(), |acc, x| {
                // Don't apply rule if the sub-order is already strict,
                // i.e. if x consists of a single TeamId
                // TODO: benchmark, possible that the allocation in the else branch is more costly.
                let new_order = if x.len() > 1 {
                    current_rule.order_group(group, x)
                } else {
                    NonStrictOrder::single(x)
                };

                acc.extend(new_order)
            });
        non_strict_group_ordering(group, remaining_rules, sub_order)
    }
}

/// Sorted list of [`TeamId`]'s
///
/// Sorted from best to worst team.
#[derive(Debug, PartialEq)]
pub struct TeamOrder(Vec<TeamId>);

impl TeamOrder {
    pub fn winner(&self) -> TeamId {
        self[OrderIdx(0)]
    }
    pub fn runner_up(&self) -> TeamId {
        self[OrderIdx(1)]
    }

    pub fn third_place(&self) -> TeamId {
        self[OrderIdx(2)]
    }

    pub fn iter(&self) -> impl Iterator<Item = &TeamId> {
        self.0.iter()
    }
}

/// Indexes [`TeamOrder`]
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct OrderIdx(pub(crate) usize);

impl IntoIterator for TeamOrder {
    type Item = TeamId;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::ops::Index<OrderIdx> for TeamOrder {
    type Output = TeamId;
    fn index(&self, idx: OrderIdx) -> &Self::Output {
        &self.0[idx.0]
    }
}

impl TryFrom<NonStrictOrder> for TeamOrder {
    type Error = GroupError;

    fn try_from(value: NonStrictOrder) -> Result<Self, Self::Error> {
        if value.is_strict() {
            Ok(TeamOrder(value.0.into_iter().map(|x| x[0]).collect()))
        } else {
            Err(GroupError::NonStrictOrder)
        }
    }
}

/// Intermediate team order representation
///
/// A non-strict team order is represented as a sorted vector of vectors of equal teams.
#[derive(Debug, PartialEq)]
pub struct NonStrictOrder(Vec<Vec<TeamId>>);

impl NonStrictOrder {
    fn single(x: Vec<TeamId>) -> Self {
        NonStrictOrder(vec![x])
    }
    fn empty() -> Self {
        NonStrictOrder(vec![])
    }

    // TODO: Did not manage to impl w/ Iterator trait.
    fn iter(&self) -> impl Iterator<Item = &Vec<TeamId>> {
        self.0.iter()
    }

    /// Initialise an equal order for a group
    ///
    /// A group with all teams equal are represented as a vector with a single element,
    /// where this element is a vector containing all the teams in the group.
    fn init_from_group(group: &Group) -> Self {
        NonStrictOrder(vec![group.team_ids().collect()])
    }

    fn init_from_teams(teams: impl Iterator<Item = TeamId>) -> Self {
        NonStrictOrder(vec![teams.collect()])
    }

    /// Strict ordering check
    ///
    /// Check if all suborders (with equal elements) are of size 1.
    /// Suborder s with |s| > 1 => non-strict ordering
    /// Suborder s with |s| < 1 (= 0) => Bug, trivial suborder are not removed correctly.
    fn is_strict(&self) -> bool {
        self.iter().all(|x| x.len() == 1)
    }

    fn extend_sub_order(mut self, team: TeamId) -> Self {
        let mut new_inner = self.0.pop().unwrap_or_default();
        new_inner.push(team);
        self.0.push(new_inner);
        self
    }

    fn add_sub_order(mut self, team: TeamId) -> Self {
        self.0.push(vec![team]);
        self
    }

    fn extend(self, sub_order: NonStrictOrder) -> Self {
        NonStrictOrder([&self.0[..], &sub_order.0[..]].concat())
    }
}

impl IntoIterator for NonStrictOrder {
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
/// [`NonStrictGroupOrder`].
pub trait SubOrdering {
    fn order_group(&self, group: &Group, order: Vec<TeamId>) -> NonStrictOrder;
    fn order_teams(
        &self,
        teams_and_groups: &HashMap<TeamId, &Group>,
        order: Vec<TeamId>,
    ) -> NonStrictOrder;
}

fn common_team_order<S: Ord + Copy>(team_stats: Vec<(TeamId, S)>) -> NonStrictOrder {
    let mut team_stats = team_stats;
    team_stats.sort_by_key(|x| x.1);
    let team_stats = team_stats;
    let (_, new_order) =
        team_stats
            .iter()
            .rev()
            .fold((team_stats[0].1, NonStrictOrder::empty()), |acc, x| {
                if acc.0 == x.1 {
                    (x.1, acc.1.extend_sub_order(x.0))
                } else {
                    (x.1, acc.1.add_sub_order(x.0))
                }
            });
    new_order
}

/// Ordering stat based on all games in the group
///
/// [`SubOrdering`] which orders by a metric based on a [`GameStat`].
/// The metric is calculated from all games in the group, regardless of the subset of teams being
/// ordered.
///
/// [`AllGroupStat`] sub-orderings based on points, goal difference and goals scored are commonly the
/// first three sub-orderings in a group rule.
struct AllGroupStat<T: GameStat>(std::marker::PhantomData<T>);

impl<T: GameStat> AllGroupStat<T> {
    fn new() -> Self {
        AllGroupStat(std::marker::PhantomData::<T>)
    }
}

impl<T: GameStat + Ord + Copy> SubOrdering for AllGroupStat<T> {
    /// Ordering for stats over the full group
    ///
    /// # Panics
    ///
    /// Does not panic since the [`GameStat::team_stats`] returns a hashmap which has all group
    /// teams as keys: The teams in `order` is a subset of the keys in `stats_all_teams`.
    fn order_group(&self, group: &Group, order: Vec<TeamId>) -> NonStrictOrder {
        // TODO: Not efficient to calc stats for all teams, but efficient is not very important
        // here.
        let stats_all_teams = T::team_stats(group);
        let team_stats = order
            .into_iter()
            .map(|id| (id, *stats_all_teams.get(&id).unwrap()))
            .collect::<Vec<(TeamId, T)>>();
        common_team_order(team_stats)
    }
    fn order_teams(
        &self,
        teams_and_groups: &HashMap<TeamId, &Group>,
        order: Vec<TeamId>,
    ) -> NonStrictOrder {
        let teams_stats = order
            .iter()
            .map(|id| {
                (
                    *id,
                    T::single_team_stats(teams_and_groups.get(id).unwrap(), *id),
                )
            })
            .collect::<Vec<(TeamId, T)>>();
        common_team_order(teams_stats)
    }
}

/// Ordering stat based on the internal games in a teams subset
///
/// SubOrdering which orders by a metric based on a `GameStat`.
/// The metric is calculated from the games in the group, where both teams involved are members of
/// the subset of teams being ordered.
struct InternalGroupStat<T: GameStat>(std::marker::PhantomData<T>);

impl<T: GameStat> InternalGroupStat<T> {
    fn new() -> Self {
        InternalGroupStat(std::marker::PhantomData::<T>)
    }
}

impl<T: GameStat + Ord + Copy> SubOrdering for InternalGroupStat<T> {
    /// Ordering for stats over internal results within the order.
    ///
    /// # Panics
    ///
    /// Does not panic since the [`GameStat::internal_team_stats`] returns a hashmap which has the internal
    /// teams as keys: The teams in `order` is equivalent to the set of keys in `internal_stats`.
    fn order_group(&self, group: &Group, order: Vec<TeamId>) -> NonStrictOrder {
        let internal_stats = T::internal_team_stats(group, &HashSet::from_iter(&order));
        let team_stats: Vec<(TeamId, T)> = order
            .into_iter()
            .map(|id| (id, *internal_stats.get(&id).unwrap()))
            .collect();
        common_team_order(team_stats)
    }
    fn order_teams(
        &self,
        _teams_and_groups: &HashMap<TeamId, &Group>,
        _order: Vec<TeamId>,
    ) -> NonStrictOrder {
        unimplemented!("Internal group stats are not used for inter-group teams ordering.")
    }
}

/// Associated with [`Rules`] to ensure strict total order.
pub trait Tiebreaker {
    fn order_teams(&self, non_strict: NonStrictOrder) -> TeamOrder {
        TeamOrder(non_strict.0.into_iter().fold(Vec::new(), |mut acc, x| {
            if x.len() == 1 {
                acc.push(x[0]);
                acc
            } else {
                [acc, self.order_sub_order(&x).0].concat()
            }
        }))
    }

    fn order_sub_order(&self, order: &[TeamId]) -> TeamOrder {
        //TODO: There must be a more efficient way to do this?
        let mut tmp_order = order.to_vec();
        tmp_order.sort_by(|a, b| self.cmp(*a, *b));
        TeamOrder(tmp_order.into_iter().rev().collect())
    }

    /// Strict comparison of teams
    ///
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
#[derive(Debug, Clone, Copy)]
pub struct Random;

impl Tiebreaker for Random {
    fn cmp(&self, _id_1: TeamId, _id_2: TeamId) -> Ordering {
        let mut rng = rand::rng();
        if rng.random::<f32>() > 0.5 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Rank tiebreaker
#[derive(Debug, Clone)]
pub struct UefaRanking(HashMap<TeamId, TeamRank>);

impl UefaRanking {
    pub fn try_new(
        groups: &Groups,
        ranking_map: HashMap<TeamId, TeamRank>,
    ) -> Result<Self, GroupError> {
        // TODO: Why does this need to be mut?
        let mut all_teams = groups.iter().flat_map(|(_, x)| x.team_ids());
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
    /// # Panics
    ///
    /// Panics if the team id's are not in `self.ranking_map`
    /// Internally ok since the fallible constructor [`UefaRanking::try_new`] ensures that the teams in the groups are a subset of the `ranking_map`.
    fn cmp(&self, id_1: TeamId, id_2: TeamId) -> Ordering {
        let rank_1 = self.0.get(&id_1).unwrap();
        let rank_2 = self.0.get(&id_2).unwrap();
        // Switch the order of comparison here so that a small rank is considered better than a
        // large one.
        rank_2.cmp(rank_1)
    }
}

/// Fifa World Cup 2018 Order
///
/// <https://www.fifa.com/worldcup/news/tie-breakers-for-russia-2018-groups>
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
pub fn fifa_2018_rules() -> Rules<Random> {
    let group_point: AllGroupStat<GroupPoint> = AllGroupStat::new();
    let goal_diff: AllGroupStat<GoalDiff> = AllGroupStat::new();
    let goal_count: AllGroupStat<GoalCount> = AllGroupStat::new();
    let int_group_point: InternalGroupStat<GroupPoint> = InternalGroupStat::new();
    let int_goal_diff: InternalGroupStat<GoalDiff> = InternalGroupStat::new();
    let int_goal_count: InternalGroupStat<GoalCount> = InternalGroupStat::new();
    let fair_play: AllGroupStat<FifaFairPlayValue> = AllGroupStat::new();
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

/// Dummy rules third-place ordering
///
/// In the Fifa world cup (at least in 2018) group third-place finishers
/// do not qualify for the playoff stage.
/// Regardless, the unified API for populating the first playoff round requires such a rule.
/// You can supply any version of rules since it will not be called anyway.
/// To avoid confusion it is nice to see that you supply rules that are not called.
///
/// It would be nice if this could be checked at compile time but I do not know how.
pub fn noop_fifa_2018_third_place_rules() -> Rules<Random> {
    Rules {
        non_strict: vec![],
        tiebreaker: Random {},
    }
}

/// Uefa Euro 2020 Order
///
/// <https://www.uefa.com/MultimediaFiles/Download/Regulations/uefaorg/Regulations/02/54/36/05/2543605_DOWNLOAD.pdf>
///
/// If two or more teams are equal on points on completion of the group matches, the following tie-breaking criteria are applied:
///
/// 1. Higher number of points obtained in the matches played between the teams in question;
/// 2. Superior goal difference resulting from the matches played between the teams in question;
/// 3. Higher number of goals scored in the matches played between the teams in question;
///
/// 4. If, after having applied criteria 1 to 3, teams still have an equal ranking, criteria 1 to 3 are reapplied exclusively to the matches between the teams who are still level to determine their final rankings.
///    (If there is a three-way tie on points, the application of the first three criteria may only break the tie for one of the teams, leaving the other two teams still tied.
///    In this case, the tiebreaking procedure is resumed, from the beginning, for the two teams that are still tied.)
///    If this procedure does not lead to a decisionr criteria 5 to 10 apply;
///
/// 5. Superior goal difference in all group matches;
/// 6. Higher number of goals scored in all group matches;
/// 7. Higher number of wins in all group matches (this criterion could only break a tie if a point deduction were to occur, as multiple teams in the same group cannot otherwise be tied on points but have a different number of wins.);
/// 8. If on the last round of the group stage, two teams are facing each other and each has the same number of points, as well as the same number of goals scored and conceded,
///    and the score finishes level in their match, their ranking is determined by a penalty shoot-out. (This criterion is not used if more than two teams have the same number of points.);
/// 9. Lower disciplinary points total in all group matches:
///     - Yellow card: -1 points
///     - Indirect red card (second yellow card): -3 points
///     - Direct red card: -3 points
///     - Yellow card and direct red card: -5 points
/// 10. Higher position in the European Qualifiers overall ranking.
/// TODO: Remaining suborderings:
/// - How to reapply 1-3 with yet another subset is unclear. I think I need to resort to impl. 1-3
/// as its own rule.
/// - The penalty shootout in 9 is pretty straightforward but needs manual data.
/// - The FairPlayValue is incorrectly calculated (of course Fifa and Uefa have different weights.)
pub fn euro_2020_rules(ranking: UefaRanking) -> Rules<UefaRanking> {
    let group_point: AllGroupStat<GroupPoint> = AllGroupStat::new();
    let int_group_point: InternalGroupStat<GroupPoint> = InternalGroupStat::new();
    let int_goal_diff: InternalGroupStat<GoalDiff> = InternalGroupStat::new();
    let int_goal_count: InternalGroupStat<GoalCount> = InternalGroupStat::new();
    let goal_diff: AllGroupStat<GoalDiff> = AllGroupStat::new();
    let num_wins: AllGroupStat<NumWins> = AllGroupStat::new();
    let fair_play: InternalGroupStat<UefaFairPlayValue> = InternalGroupStat::new();
    Rules {
        non_strict: vec![
            Box::new(group_point),
            Box::new(int_group_point),
            Box::new(int_goal_diff),
            Box::new(int_goal_count),
            Box::new(goal_diff),
            Box::new(num_wins),
            Box::new(fair_play),
        ],
        tiebreaker: ranking,
    }
}

/// Uefa Euro 2020 Third place order
pub fn euro_2020_third_place_rules(ranking: UefaRanking) -> Rules<UefaRanking> {
    let group_point: AllGroupStat<GroupPoint> = AllGroupStat::new();
    let goal_diff: AllGroupStat<GoalDiff> = AllGroupStat::new();
    let num_wins: AllGroupStat<NumWins> = AllGroupStat::new();
    let fair_play: AllGroupStat<FifaFairPlayValue> = AllGroupStat::new();
    Rules {
        non_strict: vec![
            Box::new(group_point),
            Box::new(goal_diff),
            Box::new(num_wins),
            Box::new(fair_play),
        ],
        tiebreaker: ranking,
    }
}

#[cfg(test)]
mod fifa_2018_ordering_tests {
    use super::*;
    use crate::Date;
    use crate::fair_play::{FairPlay, FairPlayScore};
    use crate::group::Group;
    use crate::group::game::PlayedGroupGame;
    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn new_point_order() {
        let game_1 =
            PlayedGroupGame::try_new(0, 0, 1, (0, 2), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_2 =
            PlayedGroupGame::try_new(1, 2, 3, (1, 1), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_3 =
            PlayedGroupGame::try_new(2, 0, 3, (0, 1), FairPlayScore::default(), Date::mock())
                .unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3]).unwrap();
        let rules = fifa_2018_rules();
        let group_order = order_group(&group, &rules);
        let true_order = TeamOrder(vec![3, 1, 2, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// Different ordering based on points vs scored goals
    #[test]
    fn points_scored_goals_discrepancy() {
        let game_1 =
            PlayedGroupGame::try_new(0, 0, 1, (0, 1), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_2 =
            PlayedGroupGame::try_new(1, 2, 3, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_3 =
            PlayedGroupGame::try_new(2, 0, 2, (0, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_4 =
            PlayedGroupGame::try_new(3, 1, 3, (5, 5), FairPlayScore::default(), Date::mock())
                .unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3, game_4]).unwrap();
        let rules = fifa_2018_rules();
        let group_order = order_group(&group, &rules);
        let true_order = TeamOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// Strict order only on PrimaryStats
    #[test]
    fn prim_stats_orders() {
        let game_1 =
            PlayedGroupGame::try_new(0, 0, 1, (0, 2), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_2 =
            PlayedGroupGame::try_new(1, 2, 3, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2]).unwrap();
        let rules = fifa_2018_rules();
        let group_order = order_group(&group, &rules);
        let true_order = TeamOrder(vec![1, 2, 3, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// One round of the group stage of 4 teams.
    /// FairPlayScore necessary for strict order.
    /// NB: The sorting is not deterministic if the order is not strict.
    #[test]
    fn fair_play_order() {
        let fair_play_home = FairPlay::new(1, 0, 0, 0);
        let fair_play_away = FairPlay::new(0, 0, 0, 0);
        let game_1 = PlayedGroupGame::try_new(
            0,
            0,
            1,
            (0, 0),
            FairPlayScore::new(fair_play_home, fair_play_away),
            Date::mock(),
        )
        .unwrap();
        let group = Group::try_new(vec![], vec![game_1]).unwrap();
        let rules = fifa_2018_rules();
        let group_order = order_group(&group, &rules);
        let true_order = TeamOrder(vec![1, 0].iter().map(|x| TeamId(*x)).collect());
        assert_eq!(true_order, group_order);
    }

    /// Two teams with same points, diff and score.
    /// The internal game decides
    #[test]
    fn internal_game() {
        let game_1 =
            PlayedGroupGame::try_new(0, 0, 2, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_2 =
            PlayedGroupGame::try_new(1, 1, 2, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_3 =
            PlayedGroupGame::try_new(2, 1, 2, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_4 =
            PlayedGroupGame::try_new(3, 0, 1, (1, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let game_5 =
            PlayedGroupGame::try_new(4, 0, 3, (0, 1), FairPlayScore::default(), Date::mock())
                .unwrap();
        let group = Group::try_new(vec![], vec![game_1, game_2, game_3, game_4, game_5]).unwrap();
        let rules = fifa_2018_rules();
        let group_order = order_group(&group, &rules);
        let true_order = TeamOrder(vec![0, 1, 3, 2].iter().map(|x| TeamId(*x)).collect());
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
        ranking.insert(TeamId(0), TeamRank(1));
        ranking.insert(TeamId(1), TeamRank(2));
        let ranking = UefaRanking(ranking);
        assert_eq!(ranking.cmp(TeamId(0), TeamId(1)), Ordering::Greater);
    }
}
