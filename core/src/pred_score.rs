//! Prediction scores
//!
//! The objective when betting on a tournament is to give accurate predictions
//! This module defines various measurements of the quality of a prediction
//!
//! NB: This module is intended as a hands-on introduction to Rust and the codebase.

// 'use' statements import code from other modules
// Imports starting with 'crate' are internal to this crate (`core`)
// External crates are listed in core/Cargo.toml
use crate::group::game::GroupGameScore;
use derive_more::{Add, AddAssign, Display, From, Into, Neg, Sub, Sum};
use serde::{Deserialize, Serialize};

// We define a `Trait`. It is the rust version of an interface.
// We say that a concrete type implements the PredScoreFn if it provides implementations of the
// methods defines in the trait.
// We will implement all different prediction score functions as different types (structs) which
// implement this trait. That way we can write generic functions and easily change the actual
// calculation of the score.
pub trait PredScoreFn {
    // By only providing a method definition, we require the implementor to implement it.
    fn pred_score(&self, pred: &GroupGameScore, truth: &GroupGameScore) -> PredScore;

    // We can also give a default impl. The implementor can override it, but doesn't have to.
    // This is useful to reduce code duplication.
    //
    // Don't worry about the content in the angle brackets for now.
    // They have to do with lifetimes and generics.
    fn group_score<T: Iterator<Item = (GroupGameScore, GroupGameScore)>>(
        &self,
        games: T,
    ) -> PredScore {
        games
            .map(|(pred, truth)| self.pred_score(&pred, &truth))
            .sum()
    }
}

// Here is an example of a concrete type that implements the `PredScoreFn` trait.
// We are free to give it any parameters (fields) we want, here the weights for the two terms in the score
// fn.
#[derive(Debug, Clone, Copy)]
pub struct SimplePredScoreFn {
    outcome_weight: f32,
    result_weight: f32,
}

// A trait is implemented by providing this type of `impl TraitX for ConcreteTypeY` block
// if this block does not implement all the methods specified (and without defaults) in the above `PredScoreFn` trait,
// the compiler will give an error.
impl PredScoreFn for SimplePredScoreFn {
    fn pred_score(&self, pred: &GroupGameScore, truth: &GroupGameScore) -> PredScore {
        // We check whether the prediction is correct both in terms of the outcome, win/draw/loose
        // and whether the exact result has been guessed.
        let correct_outcome = pred.home_outcome() == truth.home_outcome();
        let correct_result = pred == truth;
        // The variables above are bool's. We need to cast them to type f32 in order to do the
        // multiplication. There is no direct way from bool -> f32, that is why we do the casting
        // bool -> u8 -> f32
        let score = correct_outcome as u8 as f32 * self.outcome_weight
            + correct_result as u8 as f32 * self.result_weight;
        // Finally we wrap the f32 value in our type.
        // The last (non-comment) line of a rust function is returned, as long as it does not end with a ';'.
        PredScore(score)
        // we could also write:
        // return PredScore(score);
        // but it is considered un-idiomatic.
    }
}

// Perhaps you noticed that the `PredScoreFn.pred_score` function in the trait above had `PredScore` as the return type,
// and that in the impl. for `SimplePredScoreFn` we computed an f32 value `score` and then returned
// `PredScore(score)`
//
// This is a typical construct in this codebase.
// The pred. score is really represented by a floating point number (f32), but to ensure type safety we wrap it in a
// new type `PredScore` to prevent misuse. See, the README.md in the repo root for a motivation for
// this.
// The `derive` macro here is where we specify which traits we want to auto-implement for this new
// type.
// Some common derives (auto-impl's) are built-in, e.g. `Default`, `Debug` and some are provided by
// third-party crates (lib's), e.g. from `derive_more`, `serde`.
#[derive(
    Default,
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Into,
    Neg,
    From,
    PartialEq,
    PartialOrd,
    Add,
    AddAssign,
    Sub,
    Sum,
)]
pub struct PredScore(f32);

// Simple unit tests are written in the same source file,
// but still in a separate modules, which are prefaced by the `#[cfg(test)]`.
// The test modules are only compiled during testing, i.e. when running `cargo test`.
#[cfg(test)]
mod test {
    use super::*;
    use crate::group::mock_data::groups_and_teams;
    use crate::group::GroupId;
    use assert_approx_eq::assert_approx_eq;
    use itertools::Itertools;

    #[test]
    fn simple_score_fn() {
        let score_fn = SimplePredScoreFn {
            outcome_weight: 3.0,
            result_weight: 2.0,
        };
        // Correct outcome and correct result: score = 3 * 1 + 2 * 1 = 5
        let true_score = GroupGameScore::new(2, 2);
        let pred = GroupGameScore::new(2, 2);
        assert_approx_eq!(score_fn.pred_score(&pred, &true_score).0, PredScore(5.0).0);

        // Correct outcome but incorrect result: score = 3 * 1
        let true_score = GroupGameScore::new(2, 1);
        let pred = GroupGameScore::new(3, 1);
        assert_approx_eq!(score_fn.pred_score(&pred, &true_score).0, PredScore(3.0).0);
    }

    #[test]
    fn aggregating_scores() {
        let (groups, _) = groups_and_teams();
        let score_fn = SimplePredScoreFn {
            outcome_weight: 3.0,
            result_weight: 2.0,
        };
        let group_a = groups.get(&GroupId::try_from('A').unwrap()).unwrap();
        let games = group_a.played_games().map(|game| game.score);
        let (preds, trues) = games.tee();
        let games = preds.zip(trues);
        score_fn.group_score(games);
    }
}
