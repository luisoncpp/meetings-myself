use super::error::AppError;
use super::service::PlanningApp;
use chrono::{Duration, NaiveDate};
use planning_core::{
    CreateRecurringTask, CreateTask, Occurrence, OccurrenceId, Recurrence, RecurringTask,
    RecurringTaskId, Task, TaskId,
};

pub struct NewRecurringTask {
    pub title: String,
    pub recurrence: Recurrence,
}

impl PlanningApp {
    /// How far back materialization will catch up. Capped so that reopening the
    /// app after months away does not dump hundreds of stale Tasks into the Task
    /// Pool — a product decision, not an optimization.
    pub const CATCH_UP_DAYS: i64 = 31;

    pub async fn create_recurring_task(
        &self,
        request: NewRecurringTask,
    ) -> Result<RecurringTask, AppError> {
        let starts_on = self.calendar()?.today(self.clock.as_ref());
        let rule = RecurringTask::create(CreateRecurringTask {
            title: request.title,
            recurrence: request.recurrence,
            starts_on,
            clock: self.clock.as_ref(),
        })?;
        self.store(RecurringTaskId::TABLE, rule.id.as_str(), &rule)
            .await?;
        Ok(rule)
    }

    /// Generates every missing occurrence up to today. Safe to call on every app
    /// open: the `Occurrence` key makes a second attempt a no-op (A5).
    pub async fn materialize_due(&self) -> Result<Vec<Task>, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let mut produced = Vec::new();
        for rule in self.active_rules().await? {
            let made = self.materialize_rule(&rule, today).await?;
            produced.extend(made);
        }
        Ok(produced)
    }

    async fn active_rules(&self) -> Result<Vec<RecurringTask>, AppError> {
        Ok(self
            .recurring_tasks()
            .await?
            .into_iter()
            .filter(|rule| rule.lifecycle.is_active())
            .collect())
    }

    async fn materialize_rule(
        &self,
        rule: &RecurringTask,
        today: NaiveDate,
    ) -> Result<Vec<Task>, AppError> {
        let mut produced = Vec::new();
        let mut date = first_candidate(rule, today);
        while date <= today {
            if let Some(task) = self.materialize_one(rule, date).await? {
                produced.push(task);
            }
            date += Duration::days(1);
        }
        self.record_progress(rule, today).await?;
        Ok(produced)
    }

    /// Returns None when this rule does not occur on `date` or already has.
    async fn materialize_one(
        &self,
        rule: &RecurringTask,
        date: NaiveDate,
    ) -> Result<Option<Task>, AppError> {
        if !rule.recurrence.occurs_on(date) {
            return Ok(None);
        }
        let key = Occurrence::key(&rule.id, date);
        let existing: Option<Occurrence> = self.load_one(OccurrenceId::TABLE, &key).await?;
        if existing.is_some() {
            return Ok(None);
        }

        let task = Task::create(CreateTask {
            title: rule.title.clone(),
            clock: self.clock.as_ref(),
        })?;
        self.store(TaskId::TABLE, task.id.as_str(), &task).await?;

        let occurrence = Occurrence {
            id: OccurrenceId::new(key.clone()),
            rule: rule.id.clone(),
            date,
            task: task.id.clone(),
        };
        self.store(OccurrenceId::TABLE, &key, &occurrence).await?;
        Ok(Some(task))
    }

    async fn record_progress(
        &self,
        rule: &RecurringTask,
        today: NaiveDate,
    ) -> Result<(), AppError> {
        self.mutate::<RecurringTask>((RecurringTaskId::TABLE, rule.id.to_string()), |found| {
            found.materialized_through = Some(today);
        })
        .await?;
        Ok(())
    }

    pub async fn recurring_tasks(&self) -> Result<Vec<RecurringTask>, AppError> {
        self.load_all(RecurringTaskId::TABLE).await
    }
}

/// Resume from where we left off, never earlier than the rule's start and never
/// more than CATCH_UP_DAYS back.
fn first_candidate(rule: &RecurringTask, today: NaiveDate) -> NaiveDate {
    let resume = rule
        .materialized_through
        .map(|through| through + Duration::days(1))
        .unwrap_or(rule.starts_on);
    let floor = today - Duration::days(PlanningApp::CATCH_UP_DAYS);
    resume.max(rule.starts_on).max(floor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app_at;
    use chrono::{Duration, TimeZone};
    use planning_core::{FixedClock, Recurrence};
    use std::sync::Arc;
    use tempfile::TempDir;

    /// `ready_app_at` is `ready_app` from plan 0004 with a caller-supplied instant;
    /// add it to test_support.rs and express `ready_app` in terms of it.
    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(chrono::Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn reopening_the_app_never_duplicates_an_occurrence() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();

        let first = app.materialize_due().await.unwrap();
        assert_eq!(first.len(), 1);

        // Three more "app opens" on the same day.
        for _ in 0..3 {
            assert!(app.materialize_due().await.unwrap().is_empty());
        }
        assert_eq!(app.tasks().await.unwrap().len(), 1, "A5: no duplicates");
    }

    #[tokio::test]
    async fn a_gap_in_usage_catches_up_day_by_day() {
        let (_home, _drive, app, clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();
        app.materialize_due().await.unwrap();

        clock.advance(Duration::days(3));
        let caught_up = app.materialize_due().await.unwrap();
        assert_eq!(caught_up.len(), 3, "the 8th, 9th, and 10th");
        assert_eq!(app.tasks().await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn catch_up_is_capped_so_a_long_absence_does_not_flood_the_task_pool() {
        let (_home, _drive, app, clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();

        clock.advance(Duration::days(200));
        let caught_up = app.materialize_due().await.unwrap();
        assert_eq!(caught_up.len() as i64, PlanningApp::CATCH_UP_DAYS + 1);
    }

    #[tokio::test]
    async fn archiving_a_rule_stops_future_occurrences_and_keeps_past_ones() {
        let (_home, _drive, app, clock) = app_on(7).await;
        let rule = app
            .create_recurring_task(NewRecurringTask {
                title: "Morning pages".into(),
                recurrence: Recurrence::Daily,
            })
            .await
            .unwrap();
        app.materialize_due().await.unwrap();

        app.archive_recurring_task(&rule.id).await.unwrap();
        clock.advance(Duration::days(2));

        assert!(app.materialize_due().await.unwrap().is_empty());
        assert_eq!(
            app.tasks().await.unwrap().len(),
            1,
            "the occurrence already made survives"
        );
    }

    #[tokio::test]
    async fn a_materialized_occurrence_is_an_ordinary_task_unaffected_by_later_rule_edits() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let rule = app
            .create_recurring_task(NewRecurringTask {
                title: "Morning pages".into(),
                recurrence: Recurrence::Daily,
            })
            .await
            .unwrap();
        let produced = app.materialize_due().await.unwrap();
        let task = &produced[0];

        app.rename_recurring_task(&rule.id, "Evening pages".into())
            .await
            .unwrap();

        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().title,
            "Morning pages",
            "the occurrence is its own Task now"
        );
    }
}
