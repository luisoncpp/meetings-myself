use super::error::AppError;
use super::service::PlanningApp;
use super::views::{TaskState, TaskView};
use chrono::NaiveDate;
use planning_core::{
    Cadence, CalendarWeek, CheckInOutcome, Classification, HabitId, Task, TaskId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTaskView {
    pub id: TaskId,
    pub title: String,
    pub state: TaskState,
    pub importance: Classification,
    pub urgency: Classification,
    pub deadline: Option<NaiveDate>,
    pub overdue: bool,
    pub archived: bool,
    pub position: u32,
}

impl PlanTaskView {
    pub fn project(task: &Task, today: NaiveDate, position: u32) -> Self {
        let base = TaskView::project(task, today);
        Self {
            id: base.id,
            title: base.title,
            state: base.state,
            importance: base.importance,
            urgency: base.urgency,
            deadline: base.deadline,
            overdue: base.overdue,
            archived: base.archived,
            position,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanHabitView {
    pub id: HabitId,
    pub title: String,
    pub cadence: Cadence,
    pub archived: bool,
    /// True when the Habit was unpinned after this plan was made. The entry stays
    /// and stays completable — the UI shows the truth (PRODUCT.md).
    pub unpinned: bool,
    pub outcome: Option<CheckInOutcome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPlanView {
    pub date: NaiveDate,
    pub week: CalendarWeek,
    pub tasks: Vec<PlanTaskView>,
    pub habits: Vec<PlanHabitView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPoolView {
    pub focus: Vec<TaskView>,
    pub rest: Vec<TaskView>,
}

impl PlanningApp {
    pub async fn today_view(&self) -> Result<DailyPlanView, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        self.open_plan(today).await?;
        self.plan_view(today).await
    }

    /// Resolves every stored id against current entity state. An entity archived
    /// after the plan was made shows up flagged rather than disappearing.
    pub async fn plan_view(&self, date: NaiveDate) -> Result<DailyPlanView, AppError> {
        let plan = self.open_plan(date).await?;
        Ok(DailyPlanView {
            date,
            week: CalendarWeek::containing(date),
            tasks: self.project_plan_tasks(&plan).await?,
            habits: self.project_plan_habits(&plan).await?,
        })
    }

    pub async fn task_pool(&self) -> Result<TaskPoolView, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let focus = self.current_weekly_focus().await?;
        let focus_ids: HashSet<_> = focus.tasks.iter().cloned().collect();
        let tasks = self.tasks().await?;

        let poolable = |task: &Task| task.lifecycle.is_active() && !task.completion.is_complete();

        let by_id: std::collections::HashMap<_, _> =
            tasks.iter().map(|task| (task.id.clone(), task)).collect();

        let focus = focus
            .tasks
            .iter()
            .filter_map(|id| by_id.get(id))
            .filter(|task| poolable(task))
            .map(|task| TaskView::project(task, today))
            .collect();

        let rest = tasks
            .iter()
            .filter(|task| poolable(task) && !focus_ids.contains(&task.id))
            .map(|task| TaskView::project(task, today))
            .collect();

        Ok(TaskPoolView { focus, rest })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::check_in_use_cases::CheckInRequest;
    use crate::private::daily_plan_use_cases::PlanChange;
    use crate::private::habit_lifecycle::SetPinned;
    use crate::private::library::NewHabit;
    use crate::private::test_support::ready_app_at;
    use crate::private::weekly_focus_use_cases::FocusChange;
    use chrono::{TimeZone, Utc};
    use planning_core::{Cadence, DailyPlan, DailyPlanId, FixedClock};
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn a_plan_entry_whose_entity_was_archived_renders_flagged_and_ordered() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let kept = app.create_task("Keep".into()).await.unwrap();
        let archived = app.create_task("Archive me".into()).await.unwrap();
        for id in [&kept.id, &archived.id] {
            app.select_into_plan(PlanChange {
                date: today,
                task: id.clone(),
            })
            .await
            .unwrap();
        }
        app.archive_task(&archived.id).await.unwrap();

        let view = app.today_view().await.unwrap();
        assert_eq!(view.tasks.len(), 2, "the archived entry is shown, not hidden");
        assert_eq!(view.tasks[0].position, 0);
        assert_eq!(view.tasks[1].position, 1);
        let flagged = view
            .tasks
            .iter()
            .find(|task| task.id == archived.id)
            .unwrap();
        assert!(flagged.archived);
        assert_eq!(flagged.state, TaskState::Archived);
    }

    #[tokio::test]
    async fn an_unpinned_habit_still_in_todays_plan_is_flagged_and_shows_its_outcome() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        app.open_today().await.unwrap();
        app.set_habit_pinned(SetPinned {
            habit: &habit.id,
            pinned: false,
        })
        .await
        .unwrap();
        app.record_check_in(CheckInRequest {
            habit: habit.id.clone(),
            date: today,
            outcome: CheckInOutcome::Skipped,
        })
        .await
        .unwrap();

        let view = app.today_view().await.unwrap();
        assert_eq!(view.habits.len(), 1);
        assert!(view.habits[0].unpinned);
        assert_eq!(view.habits[0].outcome, Some(CheckInOutcome::Skipped));
        assert_eq!(view.week.label(), "2026-W32");
    }

    #[tokio::test]
    async fn the_task_pool_puts_weekly_focus_tasks_first_and_excludes_closed_ones() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let focused = app.create_task("Focused".into()).await.unwrap();
        let other = app.create_task("Other".into()).await.unwrap();
        let done = app.create_task("Done".into()).await.unwrap();
        let gone = app.create_task("Archived".into()).await.unwrap();
        app.add_to_focus(FocusChange {
            week,
            task: focused.id.clone(),
        })
        .await
        .unwrap();
        app.complete_task(&done.id).await.unwrap();
        app.archive_task(&gone.id).await.unwrap();

        let pool = app.task_pool().await.unwrap();
        assert_eq!(
            pool.focus.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
            vec![focused.id]
        );
        assert_eq!(
            pool.rest.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
            vec![other.id]
        );
    }

    #[tokio::test]
    async fn an_orphaned_id_in_a_plan_is_skipped_not_an_error() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let task = app.create_task("Real".into()).await.unwrap();
        app.select_into_plan(PlanChange {
            date: today,
            task: task.id.clone(),
        })
        .await
        .unwrap();

        app.mutate::<DailyPlan>((DailyPlanId::TABLE, DailyPlan::key(today)), |plan| {
            plan.tasks.push(TaskId::new("orphan"));
        })
        .await
        .unwrap();

        let view = app.plan_view(today).await.unwrap();
        assert_eq!(view.tasks.len(), 1);
        assert_eq!(view.tasks[0].id, task.id);
    }
}
