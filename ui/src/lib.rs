#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
mod app;
mod data;
mod format;
mod game;
mod group;
mod table;
mod team;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("Server error: {0}")]
    Server(String),
    #[error("Gloo error: {0}")]
    Gloo(String),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl From<gloo_net::Error> for UiError {
    fn from(err: gloo_net::Error) -> Self {
        UiError::Gloo(format!("{:?}", err))
    }
}
