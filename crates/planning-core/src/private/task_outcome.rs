use super::domain_error::DomainError;
use super::lifecycle::{Completion, Lifecycle};
use super::task::{clean_title, Task};
use chrono::NaiveDate;

/// Outcome and title changes. A one-off Task archives itself when done and
/// comes back when reopened: finished one-off work has no place in active
/// lists, yet the outcome stays on the record because the axes are orthogonal.
/// Ongoing (non-one-off) Tasks never change lifecycle here.
impl Task {
    pub fn complete(&mut self, on: NaiveDate) {
        self.completion = Completion::Completed { on };
        if self.one_off {
            self.lifecycle = Lifecycle::Archived;
        }
    }

    pub fn reopen(&mut self) {
        self.completion = Completion::Open;
        if self.one_off {
            self.lifecycle = Lifecycle::Active;
        }
    }

    /// The title that may be stored: trimmed, and never blank. Creation and
    /// rename share this rule.
    pub fn accept_title(title: String) -> Result<String, DomainError> {
        clean_title(title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use crate::private::task::CreateTask;
    use chrono::{TimeZone, Utc};

    fn task(one_off: bool) -> Task {
        Task::create(CreateTask {
            title: "Draft the letter".into(),
            one_off,
            clock: &FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap()),
        })
        .unwrap()
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    #[test]
    fn completing_a_one_off_task_archives_it_and_reopening_restores_it() {
        let mut found = task(/*one_off=*/ true);

        found.complete(day());
        assert_eq!(found.completion, Completion::Completed { on: day() });
        assert_eq!(found.lifecycle, Lifecycle::Archived);

        found.reopen();
        assert_eq!(found.completion, Completion::Open);
        assert_eq!(found.lifecycle, Lifecycle::Active);
    }

    #[test]
    fn an_ongoing_task_stays_active_through_completion() {
        let mut found = task(/*one_off=*/ false);

        found.complete(day());
        assert_eq!(found.lifecycle, Lifecycle::Active);

        found.lifecycle = Lifecycle::Archived;
        found.reopen();
        assert_eq!(
            found.lifecycle,
            Lifecycle::Archived,
            "reopen never restores an ongoing task the user archived"
        );
    }

    #[test]
    fn a_renamed_title_is_trimmed_and_never_blank() {
        assert_eq!(Task::accept_title("  Tidy  ".into()).unwrap(), "Tidy");
        assert_eq!(
            Task::accept_title("   ".into()),
            Err(DomainError::BlankTitle)
        );
    }
}
