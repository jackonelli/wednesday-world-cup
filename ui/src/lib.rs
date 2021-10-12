#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
mod app;
mod data;
mod format;
mod game;
mod group;
mod table;
mod team;

use seed::prelude::FetchError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum UiError {
    #[error("Server error: {0}")]
    Server(String),
}

/// Tmp fix until <https://github.com/seed-rs/seed/issues/544> is fixed.
impl From<FetchError> for UiError {
    fn from(err: FetchError) -> Self {
        UiError::Server(format!("{:?}", err))
    }
}
