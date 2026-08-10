use super::classification::Classification;
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::TaskId;
use super::lifecycle::{Completion, Lifecycle};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateTask<'a> {
    pub title: String,
    pub one_off: bool,
    pub clock: &'a dyn Clock,
}

fn default_one_off() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub completion: Completion,
    pub lifecycle: Lifecycle,
    pub importance: Classification,
    pub urgency: Classification,
    pub deadline: Option<NaiveDate>,
    #[serde(default = "default_one_off")]
    pub one_off: bool,
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// A title alone is enough — everything else defaults. Friction here would
    /// push capture out of the app.
    pub fn create(request: CreateTask<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: TaskId::generate(),
            title: clean_title(request.title)?,
            completion: Completion::Open,
            lifecycle: Lifecycle::Active,
            importance: Classification::Unclassified,
            urgency: Classification::Unclassified,
            deadline: None,
            one_off: request.one_off,
            created_at: request.clock.now(),
        })
    }

    /// A missed Deadline makes a Task Overdue without changing it — this is a
    /// projection, never stored (CONTEXT.md).
    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        if !self.lifecycle.is_active() || self.completion.is_complete() {
            return false;
        }
        self.deadline.is_some_and(|deadline| deadline < today)
    }
}

pub(crate) fn clean_title(title: String) -> Result<String, DomainError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankTitle);
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, day).unwrap()
    }

    #[test]
    fn a_title_alone_is_enough_to_create_a_task() {
        let task = Task::create(CreateTask {
            title: "Draft the letter".into(),
            one_off: /*one_off=*/true,
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(task.title, "Draft the letter");
        assert_eq!(task.completion, Completion::Open);
        assert_eq!(task.lifecycle, Lifecycle::Active);
        assert_eq!(task.importance, Classification::Unclassified);
        assert_eq!(task.urgency, Classification::Unclassified);
        assert_eq!(task.deadline, None);
        assert!(task.one_off);
    }

    #[test]
    fn missing_one_off_deserializes_as_true() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "title": "Pay rent",
            "completion": { "status": "open" },
            "lifecycle": "active",
            "importance": "unclassified",
            "urgency": "unclassified",
            "deadline": null,
            "createdAt": "2026-08-07T09:00:00Z"
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert!(task.one_off);
    }

    #[test]
    fn blank_titles_are_rejected_and_whitespace_is_trimmed() {
        assert_eq!(
            Task::create(CreateTask {
                title: "   ".into(),
                one_off: /*one_off=*/true,
                clock: &clock()
            }),
            Err(DomainError::BlankTitle)
        );
        let task = Task::create(CreateTask {
            title: "  Tidy  ".into(),
            one_off: /*one_off=*/true,
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(task.title, "Tidy");
    }

    #[test]
    fn a_task_is_overdue_only_while_it_is_open_active_and_past_its_deadline() {
        let mut task = Task::create(CreateTask {
            title: "File taxes".into(),
            one_off: /*one_off=*/true,
            clock: &clock(),
        })
        .unwrap();
        assert!(!task.is_overdue(day(7)), "no deadline is never overdue");

        task.deadline = Some(day(6));
        assert!(task.is_overdue(day(7)));
        assert!(
            !task.is_overdue(day(6)),
            "the deadline day itself is not yet overdue"
        );

        task.completion = Completion::Completed { on: day(7) };
        assert!(!task.is_overdue(day(7)), "completed work is not overdue");

        task.completion = Completion::Open;
        task.lifecycle = Lifecycle::Archived;
        assert!(
            !task.is_overdue(day(7)),
            "archived work is not actionable, so not overdue"
        );
    }
}
