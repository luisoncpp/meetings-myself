use planning_store::{StoreError, StoreHealth};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error("the synchronized data is not ready: {0:?}")]
    NotReady(StoreHealth),

    #[error("no synchronization folder has been chosen yet")]
    NoDatabase,

    #[error("'{0}' is not an IANA time zone")]
    InvalidZone(String),
}
