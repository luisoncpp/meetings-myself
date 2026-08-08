use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{CalendarWeek, StartFocus, TaskId, WeeklyFocus, WeeklyFocusId};

pub struct FocusChange {
    pub week: CalendarWeek,
    pub task: TaskId,
}

impl PlanningApp {
    /// Creates an empty focus on first read so callers never branch on "does one exist".
    pub async fn weekly_focus(&self, week: CalendarWeek) -> Result<WeeklyFocus, AppError> {
        let key = WeeklyFocus::key(week);
        if let Some(found) = self
            .load_one::<WeeklyFocus>(WeeklyFocusId::TABLE, &key)
            .await?
        {
            return Ok(found);
        }
        let created = WeeklyFocus::start(StartFocus {
            week,
            clock: self.clock.as_ref(),
        });
        self.store(WeeklyFocusId::TABLE, &key, &created).await?;
        Ok(created)
    }

    pub async fn current_weekly_focus(&self) -> Result<WeeklyFocus, AppError> {
        let week = self.calendar()?.current_week(self.clock.as_ref());
        self.weekly_focus(week).await
    }

    /// Archived Tasks cannot be newly selected, but ones already present stay —
    /// this method is the "newly" half of that rule (ADR 0002).
    pub async fn add_to_focus(&self, change: FocusChange) -> Result<(), AppError> {
        self.require_selectable_task(&change.task).await?;
        let key = WeeklyFocus::key(change.week);
        self.weekly_focus(change.week).await?;
        self.mutate::<WeeklyFocus>((WeeklyFocusId::TABLE, key), |focus| {
            focus.add(change.task.clone());
        })
        .await?;
        Ok(())
    }

    pub async fn remove_from_focus(&self, change: FocusChange) -> Result<(), AppError> {
        let key = WeeklyFocus::key(change.week);
        self.mutate::<WeeklyFocus>((WeeklyFocusId::TABLE, key), |focus| {
            focus.remove(&change.task);
        })
        .await?;
        Ok(())
    }

    pub(crate) async fn require_selectable_task(&self, task: &TaskId) -> Result<(), AppError> {
        let found = self.task(task).await?.ok_or(AppError::NotFound {
            table: "task",
            id: task.to_string(),
        })?;
        if !found.lifecycle.is_active() {
            return Err(AppError::NotSelectable {
                reason: "the task is archived",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app_at;
    use chrono::{TimeZone, Utc};
    use planning_core::FixedClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn a_focus_is_created_empty_on_first_read_and_survives_reload() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        assert!(app.weekly_focus(week).await.unwrap().tasks.is_empty());

        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.add_to_focus(FocusChange {
            week,
            task: task.id.clone(),
        })
        .await
        .unwrap();
        assert_eq!(app.weekly_focus(week).await.unwrap().tasks, vec![task.id]);
    }

    #[tokio::test]
    async fn selecting_into_a_focus_never_removes_the_task_from_the_pool() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.add_to_focus(FocusChange {
            week,
            task: task.id.clone(),
        })
        .await
        .unwrap();

        assert_eq!(app.tasks().await.unwrap().len(), 1);
        assert!(app
            .task(&task.id)
            .await
            .unwrap()
            .unwrap()
            .lifecycle
            .is_active());
    }

    #[tokio::test]
    async fn an_archived_task_cannot_be_newly_selected_into_a_focus() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let task = app.create_task("Old idea".into()).await.unwrap();
        app.archive_task(&task.id).await.unwrap();

        assert!(matches!(
            app.add_to_focus(FocusChange {
                week,
                task: task.id
            })
            .await
            .unwrap_err(),
            AppError::NotSelectable { .. }
        ));
    }

    #[tokio::test]
    async fn archiving_a_task_already_in_a_focus_leaves_the_entry_in_place() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.add_to_focus(FocusChange {
            week,
            task: task.id.clone(),
        })
        .await
        .unwrap();

        app.archive_task(&task.id).await.unwrap();

        // Forward-only: the focus is not rewritten (ADR 0002).
        assert_eq!(app.weekly_focus(week).await.unwrap().tasks, vec![task.id]);
    }
}
