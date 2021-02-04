//! # Wednesday world cup (*wwc*) core library
//!
//! ## Functional
//! The library use functional paradigms wherever possible.
//! The actual data structures are very simple. Derived values, metrics et c. are calculated from
//! simpler data and is not represented in itself. A good example of this is the
//! [`Group`](group/struct.Group.html) struct which only stores data about the score in each group
//! game. Teams, winner, the points of a certain team is not stored, or even cached, but rather
//! derived from the played games.
//! This makes for a very clean and composable API with consistent results. The down-side is of course that calculations
//! are repeated unecessarily, but then again, the size of the average tournament is very small and
//! the overhead will be miniscule.
#![forbid(unsafe_code)]
#![feature(proc_macro_hygiene, decl_macro)]
// Enable clippy if our Cargo.toml file asked us to do so.
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(
    //missing_copy_implementations,
    //missing_debug_implementations,
    //TODO Enable
    //missing_docs,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(feature = "clippy", warn(cast_possible_truncation))]
#![cfg_attr(feature = "clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature = "clippy", warn(cast_precision_loss))]
#![cfg_attr(feature = "clippy", warn(cast_sign_loss))]
#![cfg_attr(feature = "clippy", warn(missing_docs_in_private_items))]
#![cfg_attr(feature = "clippy", warn(mut_mut))]
// Disallow `println!`. Use `debug!` for debug output
// (which is provided by the `log` crate).
#![cfg_attr(feature = "clippy", warn(print_stdout))]
// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature = "clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature = "clippy", warn(unseparated_literal_suffix))]
#![cfg_attr(feature = "clippy", warn(wrong_pub_self_convention))]

pub mod error;
pub mod fair_play;
pub mod game;
pub mod group;
// pub mod playoff;
pub mod player;
pub mod pred_score;
pub mod team;
pub mod utils;
// Exports
pub use team::Team;
pub use utils::date::Date;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// TODO: Is this necessary in the core crate? I'd rather move this to wasm crates which includes
// this. I think it should work that way but I'm not sure.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
