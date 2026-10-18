use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{Task, TaskId};

impl PlanningApp {
    pub async fn complete_task(&self, task: &TaskId) -> Result<(), AppError> {
        let on = self.today()?;
        self.complete_task_on(task, on).await
    }

    /// `on` is the calendar day the outcome belongs to (yesterday catch-up
    /// stamps yesterday). Dates after home-zone today are refused. One-off
    /// Tasks archive in the same write (`Task::complete`).
    pub async fn complete_task_on(&self, task: &TaskId, on: NaiveDate) -> Result<(), AppError> {
        if on > self.today()? {
            return Err(AppError::FutureCompletion);
        }
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| {
            found.complete(on)
        })
        .await?;
        Ok(())
    }

    /// Recorded outcomes stay correctable at any time (ADR 0002) — and completing
    /// a Task was never gated on a Daily Plan, so reopening is not either.
    pub async fn reopen_task(&self, task: &TaskId) -> Result<(), AppError> {
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), Task::reopen)
            .await?;
        Ok(())
    }

    /// Plans, pools and summaries read the title when they project, so a rename
    /// shows everywhere on the next view.
    pub async fn rename_task(&self, task: &TaskId, title: String) -> Result<(), AppError> {
        let title = Task::accept_title(title)?;
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| {
            found.title = title;
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::{ready_app, ready_app_at};
    use crate::private::views::LibraryFilter;
    use chrono::{Duration, TimeZone, Utc};
    use planning_core::{Completion, DomainError, Lifecycle};

    #[tokio::test]
    async fn completion_is_reversible_and_dated_in_the_home_zone() {
        let (_home, _drive, app) = ready_app().await;
        let task = app
            .create_task("Draft the letter".into(), /*one_off=*/ true)
            .await
            .unwrap();

        app.complete_task(&task.id).await.unwrap();
        let completed = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(
            completed.completion,
            Completion::Completed {
                on: NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
            }
        );

        app.reopen_task(&task.id).await.unwrap();
        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().completion,
            Completion::Open
        );
    }

    #[tokio::test]
    async fn a_done_one_off_task_leaves_the_active_library_and_reopening_brings_it_back() {
        let (_home, _drive, app) = ready_app().await;
        let task = app
            .create_task("Send invoice".into(), /*one_off=*/ true)
            .await
            .unwrap();
        let active = LibraryFilter {
            include_archived: false,
        };

        app.complete_task(&task.id).await.unwrap();
        assert!(app.library(active).await.unwrap().tasks.is_empty());
        let stored = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(stored.lifecycle, Lifecycle::Archived);

        app.reopen_task(&task.id).await.unwrap();
        let stored = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(stored.lifecycle, Lifecycle::Active);
    }

    #[tokio::test]
    async fn completing_on_a_past_day_stamps_that_day_not_today() {
        let (_home, _drive, app, _clock) =
            ready_app_at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap()).await;
        let task = app
            .create_task("Catch up".into(), /*one_off=*/ true)
            .await
            .unwrap();
        let yesterday = app.calendar().unwrap().today(app.clock_ref()) - Duration::days(1);

        app.complete_task_on(&task.id, yesterday).await.unwrap();
        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().completion,
            Completion::Completed { on: yesterday }
        );
    }

    #[tokio::test]
    async fn completing_on_a_future_day_is_refused() {
        let (_home, _drive, app, _clock) =
            ready_app_at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap()).await;
        let task = app
            .create_task("Tomorrow".into(), /*one_off=*/ true)
            .await
            .unwrap();
        let tomorrow = app.calendar().unwrap().today(app.clock_ref()) + Duration::days(1);

        let error = app.complete_task_on(&task.id, tomorrow).await.unwrap_err();
        assert!(matches!(error, AppError::FutureCompletion));
        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().completion,
            Completion::Open
        );
    }

    #[tokio::test]
    async fn renaming_a_task_trims_it_and_refuses_blank_titles() {
        let (_home, _drive, app) = ready_app().await;
        let task = app
            .create_task("Draft".into(), /*one_off=*/ true)
            .await
            .unwrap();

        app.rename_task(&task.id, "  Draft the post  ".into())
            .await
            .unwrap();
        let error = app.rename_task(&task.id, "  ".into()).await.unwrap_err();

        assert!(matches!(error, AppError::Domain(DomainError::BlankTitle)));
        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().title,
            "Draft the post"
        );
    }
}
