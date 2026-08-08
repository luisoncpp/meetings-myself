use planning_core::DomainError;
use planning_reports::ReportError;
use planning_store::{StoreError, StoreHealth};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Report(#[from] ReportError),

    #[error("the synchronized data is not ready: {0:?}")]
    NotReady(StoreHealth),

    #[error("no synchronization folder has been chosen yet")]
    NoDatabase,

    #[error("no {table} with id {id}")]
    NotFound { table: &'static str, id: String },

    #[error("cannot select: {reason}")]
    NotSelectable { reason: &'static str },

    #[error("the proposed order is not a permutation of the plan")]
    InvalidOrder,

    #[error("'{0}' is not an IANA time zone")]
    InvalidZone(String),
}
