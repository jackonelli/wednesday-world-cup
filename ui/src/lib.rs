#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
mod app;
mod data;
mod game;
mod group;
mod playoff;
mod table;
mod team;

use leptos::prelude::*;
use thiserror::Error;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <app::App/> })
}
