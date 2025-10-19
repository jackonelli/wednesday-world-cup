pub mod sqlx_impl;

use thiserror::Error;
use wwc_core::error::WwcError;

// Re-export main SQLx implementation
pub use sqlx_impl::*;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Missing 'DATABASE_URL'")]
    DbUrlMissing,
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Core error: {0}")]
    Core(#[from] WwcError),
    #[error("Could you be more specific: {0}")]
    Generic(String),
}
