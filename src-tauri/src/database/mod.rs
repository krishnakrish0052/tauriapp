pub mod postgres;
pub mod models;
pub mod sync;
pub mod shared;

pub use postgres::DatabaseManager;
pub use models::*;
pub use sync::*;
pub use shared::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Insufficient credits")]
    InsufficientCredits,
    #[error("Invalid session status: {0}")]
    InvalidSessionStatus(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
