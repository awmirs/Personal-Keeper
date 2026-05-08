use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Sync error: {0}")]
    Sync(String),
}