use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::{OccurrenceId, RecurringTaskId, TaskId};
use super::lifecycle::Lifecycle;
use super::recurrence::Recurrence;
use super::task::clean_title;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateRecurringTask<'a> {
    pub title: String,
    pub recurrence: Recurrence,
    pub starts_on: NaiveDate,
    pub clock: &'a dyn Clock,
}

/// A factory for Tasks, not a Task. Archiving it stops future materialization and
/// leaves every occurrence already produced untouched (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTask {
    pub id: RecurringTaskId,
    pub title: String,
    pub recurrence: Recurrence,
    pub lifecycle: Lifecycle,
    pub starts_on: NaiveDate,
    /// Fast path only. Correctness comes from `Occurrence`'s key, not from this.
    pub materialized_through: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl RecurringTask {
    pub fn create(request: CreateRecurringTask<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: RecurringTaskId::generate(),
            title: clean_title(request.title)?,
            recurrence: request.recurrence,
            lifecycle: Lifecycle::Active,
            starts_on: request.starts_on,
            materialized_through: None,
            created_at: request.clock.now(),
        })
    }
}

/// Proof that one rule produced one Task on one date. Its key makes a duplicate
/// structurally impossible, so reopening the app cannot double-generate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    pub id: OccurrenceId,
    pub rule: RecurringTaskId,
    pub date: NaiveDate,
    pub task: TaskId,
}

impl Occurrence {
    pub fn key(rule: &RecurringTaskId, date: NaiveDate) -> String {
        format!("{rule}:{}", date.format("%Y-%m-%d"))
    }
}
