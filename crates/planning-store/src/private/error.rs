use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("could not read or write {0}")]
    Io(#[from] std::io::Error),

    #[error("{path} is not readable as settings: {detail}")]
    Corrupt { path: PathBuf, detail: String },

    #[error("this operating system reported no configuration directory")]
    NoConfigDirectory,

    #[error("database error: {0}")]
    Database(String),
}

impl From<surrealdb::Error> for StoreError {
    fn from(error: surrealdb::Error) -> Self {
        StoreError::Database(error.to_string())
    }
}
