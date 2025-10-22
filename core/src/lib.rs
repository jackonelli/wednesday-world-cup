//! # Wednesday world cup (*wwc*) core library
//!
//! ## Functional
//! The library uses functional paradigms wherever possible.
//! The actual data structures are very simple. Derived values, metrics et c. are calculated from
//! simpler data and is not represented in itself. A good example of this is the
//! [`Group`](group/struct.Group.html) struct which only stores a list of games for each group.
//! Teams, winner, the points of a certain team is not stored, or even cached, but rather
//! derived from the played games.
//! This makes for a very clean and composable API with consistent results. The down-side is of course that calculations
//! are repeated unecessarily, but then again, the size of the average tournament is very small and
//! the overhead will be miniscule.
#![forbid(unsafe_code)]
// Enable as many useful Rust and Clippy warnings as we can stand.  We'd
// also enable `trivial_casts`, but we're waiting for
// https://github.com/rust-lang/rust/issues/23416.
#![warn(
    missing_copy_implementations,
    //missing_debug_implementations,
    //TODO Enable
    //missing_docs,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_possible_wrap)]
#![warn(clippy::cast_precision_loss)]
#![warn(clippy::cast_sign_loss)]
// #![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::mut_mut)]
// Disallow `println!`. Use `debug!` for debug output
// (which is provided by the `log` crate).
#![warn(clippy::print_stdout)]
// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![warn(clippy::unseparated_literal_suffix)]

pub mod error;
pub mod fair_play;
pub mod game;
pub mod group;
pub mod player;
pub mod playoff;
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
// Note: wee_alloc feature is not currently defined in Cargo.toml
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
