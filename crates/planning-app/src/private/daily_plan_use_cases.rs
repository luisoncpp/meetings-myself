use super::error::AppError;
use super::service::PlanningApp;
use chrono::{Datelike, NaiveDate};
use planning_core::{DailyPlan, DailyPlanId, HabitId, StartPlan, Task, TaskId};

pub struct PlanChange {
    pub date: NaiveDate,
    pub task: TaskId,
}

pub struct PlanHabitChange {
    pub date: NaiveDate,
    pub habit: HabitId,
}

pub struct ReorderPlan {
    pub date: NaiveDate,
    pub order: Vec<TaskId>,
}

impl PlanningApp {
    /// Materializes any due Recurring Tasks first so the Task Pool is complete
    /// before the plan is seeded.
    pub async fn open_today(&self) -> Result<DailyPlan, AppError> {
        self.materialize_due().await?;
        let today = self.calendar()?.today(self.clock.as_ref());
        self.open_plan(today).await
    }

    /// Creates the plan on first open and returns it untouched afterwards.
    /// Re-seeding an existing plan would rewrite the user's own selection.
    pub async fn open_plan(&self, date: NaiveDate) -> Result<DailyPlan, AppError> {
        let key = DailyPlan::key(date);
        if let Some(found) = self.load_one::<DailyPlan>(DailyPlanId::TABLE, &key).await? {
            return Ok(found);
        }
        let habits = self.habits_due_on(date).await?;
        let created = DailyPlan::start(StartPlan {
            date,
            habits,
            clock: self.clock.as_ref(),
        });
        self.store(DailyPlanId::TABLE, &key, &created).await?;
        Ok(created)
    }

    /// Read-only existence check. Plan 0008's launcher calls exactly this and must
    /// never cause a plan to be created as a side effect.
    pub async fn has_plan_for(&self, date: NaiveDate) -> Result<bool, AppError> {
        let found: Option<DailyPlan> = self
            .load_one(DailyPlanId::TABLE, &DailyPlan::key(date))
            .await?;
        Ok(found.is_some())
    }

    /// Pinned, active, and due today. Unpinning and cadence changes therefore take
    /// effect from the next plan, never the current one (ADR 0002).
    async fn habits_due_on(&self, date: NaiveDate) -> Result<Vec<HabitId>, AppError> {
        Ok(self
            .habits()
            .await?
            .into_iter()
            .filter(|habit| habit.pinned && habit.lifecycle.is_active())
            .filter(|habit| habit.cadence.is_due(date.weekday()))
            .map(|habit| habit.id)
            .collect())
    }

    pub async fn select_into_plan(&self, change: PlanChange) -> Result<(), AppError> {
        self.require_selectable_task(&change.task).await?;
        self.open_plan(change.date).await?;
        self.mutate::<DailyPlan>((DailyPlanId::TABLE, DailyPlan::key(change.date)), |plan| {
            plan.select(change.task.clone());
        })
        .await?;
        Ok(())
    }

    pub async fn remove_from_plan(&self, change: PlanChange) -> Result<(), AppError> {
        self.mutate::<DailyPlan>((DailyPlanId::TABLE, DailyPlan::key(change.date)), |plan| {
            plan.unselect(&change.task);
        })
        .await?;
        Ok(())
    }

    pub async fn reorder_plan(&self, request: ReorderPlan) -> Result<(), AppError> {
        let key = DailyPlan::key(request.date);
        let mut plan: DailyPlan =
            self.load_one(DailyPlanId::TABLE, &key)
                .await?
                .ok_or(AppError::NotFound {
                    table: "daily_plan",
                    id: key.clone(),
                })?;
        if !plan.reorder(request.order) {
            return Err(AppError::InvalidOrder);
        }
        self.store(DailyPlanId::TABLE, &key, &plan).await?;
        Ok(())
    }

    /// The manual one-day addition: a Habit that is not pinned, or not due today,
    /// can still be added to this one plan.
    pub async fn add_habit_to_plan(&self, change: PlanHabitChange) -> Result<(), AppError> {
        self.open_plan(change.date).await?;
        self.mutate::<DailyPlan>((DailyPlanId::TABLE, DailyPlan::key(change.date)), |plan| {
            plan.include_habit(change.habit.clone());
        })
        .await?;
        Ok(())
    }

    /// The Daily Plan's contextual shortcut: create and select in one action.
    pub async fn quick_add_task(&self, title: String) -> Result<Task, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let task = self.create_task(title).await?;
        self.select_into_plan(PlanChange {
            date: today,
            task: task.id.clone(),
        })
        .await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::habit_lifecycle::SetPinned;
    use crate::private::library::NewHabit;
    use crate::private::test_support::ready_app_at;
    use chrono::{Duration, TimeZone, Utc, Weekday};
    use planning_core::{Cadence, FixedClock};
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn opening_twice_returns_the_same_plan_without_reseeding() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();

        let first = app.open_today().await.unwrap();
        assert_eq!(first.habits, vec![habit.id.clone()]);

        app.set_habit_pinned(SetPinned {
            habit: &habit.id,
            pinned: false,
        })
        .await
        .unwrap();
        let second = app.open_today().await.unwrap();
        assert_eq!(
            second.habits,
            vec![habit.id],
            "forward-only: today's plan is not rewritten"
        );
    }

    #[tokio::test]
    async fn pinned_habits_join_only_on_their_cadence_days() {
        // 2026-08-07 is a Friday; 2026-08-08 is a Saturday.
        let (_home, _drive, app, clock) = app_on(7).await;
        let weekday_only = app
            .create_habit(NewHabit {
                title: "Deep work".into(),
                cadence: Cadence::new_on_weekdays(&[Weekday::Fri]).unwrap(),
            })
            .await
            .unwrap();

        assert_eq!(
            app.open_today().await.unwrap().habits,
            vec![weekday_only.id.clone()]
        );

        clock.advance(Duration::days(1));
        assert!(
            app.open_today().await.unwrap().habits.is_empty(),
            "Saturday is not a cadence day"
        );
    }

    #[tokio::test]
    async fn an_unpinned_habit_is_never_seeded_into_a_new_plan() {
        let (_home, _drive, app, clock) = app_on(7).await;
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        app.set_habit_pinned(SetPinned {
            habit: &habit.id,
            pinned: false,
        })
        .await
        .unwrap();

        clock.advance(Duration::days(1));
        assert!(app.open_today().await.unwrap().habits.is_empty());
    }

    #[tokio::test]
    async fn tasks_can_be_selected_ordered_removed_and_completed_without_duplicating() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let mut ids = Vec::new();
        for title in ["One", "Two", "Three"] {
            ids.push(app.create_task(title.into()).await.unwrap().id);
        }

        for id in &ids {
            app.select_into_plan(PlanChange {
                date: today,
                task: id.clone(),
            })
            .await
            .unwrap();
        }
        // Selecting twice is a no-op, not a duplicate (A2).
        app.select_into_plan(PlanChange {
            date: today,
            task: ids[0].clone(),
        })
        .await
        .unwrap();
        assert_eq!(app.open_today().await.unwrap().tasks.len(), 3);

        app.reorder_plan(ReorderPlan {
            date: today,
            order: vec![ids[2].clone(), ids[0].clone(), ids[1].clone()],
        })
        .await
        .unwrap();
        assert_eq!(app.open_today().await.unwrap().tasks[0], ids[2]);

        app.remove_from_plan(PlanChange {
            date: today,
            task: ids[1].clone(),
        })
        .await
        .unwrap();
        assert_eq!(app.open_today().await.unwrap().tasks.len(), 2);
        assert_eq!(
            app.tasks().await.unwrap().len(),
            3,
            "removal never touches the Task Pool"
        );

        app.complete_task(&ids[0]).await.unwrap();
        assert!(app
            .task(&ids[0])
            .await
            .unwrap()
            .unwrap()
            .completion
            .is_complete());
    }

    #[tokio::test]
    async fn a_bad_reorder_is_rejected_without_changing_the_plan() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let task = app.create_task("One".into()).await.unwrap();
        app.select_into_plan(PlanChange {
            date: today,
            task: task.id.clone(),
        })
        .await
        .unwrap();

        let error = app
            .reorder_plan(ReorderPlan {
                date: today,
                order: vec![TaskId::new("ghost")],
            })
            .await
            .unwrap_err();
        assert!(matches!(error, AppError::InvalidOrder));
        assert_eq!(app.open_today().await.unwrap().tasks, vec![task.id]);
    }

    /// A6, second half.
    #[tokio::test]
    async fn archiving_an_entry_already_in_a_plan_leaves_it_in_place_and_completable() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.select_into_plan(PlanChange {
            date: today,
            task: task.id.clone(),
        })
        .await
        .unwrap();

        app.archive_task(&task.id).await.unwrap();

        let plan = app.open_today().await.unwrap();
        assert_eq!(plan.tasks, vec![task.id.clone()], "the entry stays");

        // Still completable while archived — completion is never gated on the plan.
        app.complete_task(&task.id).await.unwrap();
        assert!(app
            .task(&task.id)
            .await
            .unwrap()
            .unwrap()
            .completion
            .is_complete());

        // But it cannot be newly selected into another day.
        let tomorrow = today + Duration::days(1);
        assert!(matches!(
            app.select_into_plan(PlanChange {
                date: tomorrow,
                task: task.id
            })
            .await
            .unwrap_err(),
            AppError::NotSelectable { .. }
        ));
    }

    #[tokio::test]
    async fn quick_add_creates_the_task_and_selects_it_into_today() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let task = app.quick_add_task("Call the bank".into()).await.unwrap();
        assert_eq!(app.open_today().await.unwrap().tasks, vec![task.id]);
    }

    #[tokio::test]
    async fn has_plan_for_reports_without_creating_one() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        assert!(!app.has_plan_for(today).await.unwrap());
        assert!(
            !app.has_plan_for(today).await.unwrap(),
            "asking must not create a plan"
        );

        app.open_today().await.unwrap();
        assert!(app.has_plan_for(today).await.unwrap());
    }
}
