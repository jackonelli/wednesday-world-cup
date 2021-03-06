//! Prediction scores
//!
//! The objective when betting on a tournament is to give accurate predictions
//! This module defines various measurements of the quality of a prediction
use crate::game::Score;
use derive_more::{Add, AddAssign, Display, From, Into, Neg, Sub};
use serde::{Deserialize, Serialize};

// We define a `Trait`. It is the rust version of an interface.
// We say that a concrete type implements the PredScoreFn if it provides a specific `pred_score`
// function.
// We will implement all different prediction score functions as different types (structs) which
// implement this trait. That way we can write generic functions and easily change the actual
// calculation of the score.
pub trait PredScoreFn {
    fn pred_score(&self, pred: Score, truth: Score) -> PredScore;
}

// Here is an example of a concrete type that implements the `PredScoreFn` trait.
// We are free to give it any parameters we want, here the weights for the two terms in the score
// fn.
#[derive(Debug, Clone, Copy)]
pub struct SimplePredScoreFn {
    outcome: f32,
    result: f32,
}

// A trait is implemented by providing this type of `impl TraitX for ConcreteTypeY` block
// if this block does not implement all the functions specified in the above `PredScoreFn` trait,
// the compiler will give an error.
impl PredScoreFn for SimplePredScoreFn {
    fn pred_score(&self, pred: Score, truth: Score) -> PredScore {
        // We check whether the prediction is correct both in terms of the outcome, win/draw/loose
        // and whether the exact result has been guessed.
        let correct_outcome = pred.home_outcome() == truth.home_outcome();
        let correct_result = pred == truth;
        // The variables above are bool's. We need to cast them to type f32 in order to do the
        // multiplication. There is no direct way from bool -> f32, that is why we do the casting
        // bool -> u8 -> f32
        let score =
            correct_outcome as u8 as f32 * self.outcome + correct_result as u8 as f32 * self.result;
        // Finally we wrap the f32 value in our type.
        // The last (non-comment) line of a rust function is returned, as long as it does not end with a ';'.
        PredScore(score)
        // we could also write:
        // return PredScore(score);
        // but it is considered un-idiomatic.
    }
}

// This is a typical construct in this code.
// The pred. score is really represented by a floating number, but to ensure type safety we wrap it in a
// new type `PredScore` to prevent misuse.
// The `derive` macro here is where we specify which traits we want to auto-implement for this new
// type.
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
)]
pub struct PredScore(f32);

#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_simple_score_fn() {
        let score_fn = SimplePredScoreFn {
            outcome: 3.0,
            result: 2.0,
        };
        // Correct outcome and correct result: score = 3 * 1 + 2 * 1
        let true_score = Score::new(2, 2);
        let pred = Score::new(2, 2);
        assert_approx_eq!(score_fn.pred_score(pred, true_score).0, PredScore(5.0).0);

        // Correct outcome but incorrect result: score = 3 * 1
        let true_score = Score::new(2, 1);
        let pred = Score::new(3, 1);
        assert_approx_eq!(score_fn.pred_score(pred, true_score).0, PredScore(3.0).0);
    }
}
