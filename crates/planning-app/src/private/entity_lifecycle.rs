use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{
    Achievement, Classification, Completion, Goal, GoalId, Lifecycle, Task, TaskId,
};

pub struct ClassifyTask<'a> {
    pub task: &'a TaskId,
    pub importance: Classification,
    pub urgency: Classification,
}

pub struct SetDeadline<'a> {
    pub task: &'a TaskId,
    pub deadline: Option<NaiveDate>,
}

impl PlanningApp {
    fn today(&self) -> Result<NaiveDate, AppError> {
        Ok(self.calendar()?.today(self.clock.as_ref()))
    }

    pub async fn task(&self, task: &TaskId) -> Result<Option<Task>, AppError> {
        self.load_one(TaskId::TABLE, task.as_str()).await
    }

    pub async fn goal(&self, goal: &GoalId) -> Result<Option<Goal>, AppError> {
        self.load_one(GoalId::TABLE, goal.as_str()).await
    }

    pub async fn complete_task(&self, task: &TaskId) -> Result<(), AppError> {
        let on = self.today()?;
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| {
            found.completion = Completion::Completed { on };
        })
        .await?;
        Ok(())
    }

    /// Recorded outcomes stay correctable at any time (ADR 0002) — and completing
    /// a Task was never gated on a Daily Plan, so reopening is not either.
    pub async fn reopen_task(&self, task: &TaskId) -> Result<(), AppError> {
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| {
            found.completion = Completion::Open;
        })
        .await?;
        Ok(())
    }

    pub async fn archive_task(&self, task: &TaskId) -> Result<(), AppError> {
        self.set_task_lifecycle(task, Lifecycle::Archived).await
    }

    pub async fn restore_task(&self, task: &TaskId) -> Result<(), AppError> {
        self.set_task_lifecycle(task, Lifecycle::Active).await
    }

    async fn set_task_lifecycle(&self, task: &TaskId, to: Lifecycle) -> Result<(), AppError> {
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| {
            found.lifecycle = to
        })
        .await?;
        Ok(())
    }

    pub async fn achieve_goal(&self, goal: &GoalId) -> Result<(), AppError> {
        let on = self.today()?;
        self.mutate::<Goal>((GoalId::TABLE, goal.to_string()), |found| {
            found.achievement = Achievement::Achieved { on };
        })
        .await?;
        Ok(())
    }

    pub async fn unachieve_goal(&self, goal: &GoalId) -> Result<(), AppError> {
        self.mutate::<Goal>((GoalId::TABLE, goal.to_string()), |found| {
            found.achievement = Achievement::Pursuing;
        })
        .await?;
        Ok(())
    }

    pub async fn archive_goal(&self, goal: &GoalId) -> Result<(), AppError> {
        self.set_goal_lifecycle(goal, Lifecycle::Archived).await
    }

    pub async fn restore_goal(&self, goal: &GoalId) -> Result<(), AppError> {
        self.set_goal_lifecycle(goal, Lifecycle::Active).await
    }

    async fn set_goal_lifecycle(&self, goal: &GoalId, to: Lifecycle) -> Result<(), AppError> {
        self.mutate::<Goal>((GoalId::TABLE, goal.to_string()), |found| {
            found.lifecycle = to
        })
        .await?;
        Ok(())
    }

    pub async fn set_task_classification(&self, request: ClassifyTask<'_>) -> Result<(), AppError> {
        self.mutate::<Task>((TaskId::TABLE, request.task.to_string()), |found| {
            found.importance = request.importance;
            found.urgency = request.urgency;
        })
        .await?;
        Ok(())
    }

    pub async fn set_task_deadline(&self, request: SetDeadline<'_>) -> Result<(), AppError> {
        self.mutate::<Task>((TaskId::TABLE, request.task.to_string()), |found| {
            found.deadline = request.deadline;
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::habit_lifecycle::SetCadence;
    use crate::private::library::{NewGoal, NewHabit};
    use crate::private::test_support::ready_app;
    use chrono::{NaiveDate, Weekday};
    use planning_core::{Cadence, TaskId};

    #[tokio::test]
    async fn archiving_is_reversible_and_preserves_completion() {
        let (_home, _drive, app) = ready_app().await;
        let task = app.create_task("File taxes".into()).await.unwrap();

        app.complete_task(&task.id).await.unwrap();
        app.archive_task(&task.id).await.unwrap();

        let archived = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(archived.lifecycle, Lifecycle::Archived);
        assert!(
            archived.completion.is_complete(),
            "archiving must not erase the outcome"
        );

        app.restore_task(&task.id).await.unwrap();
        let restored = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(restored.lifecycle, Lifecycle::Active);
        assert!(
            restored.completion.is_complete(),
            "restoring returns it exactly as it was"
        );
    }

    #[tokio::test]
    async fn completion_is_reversible_and_dated_in_the_home_zone() {
        let (_home, _drive, app) = ready_app().await;
        let task = app.create_task("Draft the letter".into()).await.unwrap();

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
    async fn goals_are_achievable_and_un_achievable() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app
            .create_goal(NewGoal {
                title: "Career transition".into(),
                target_date: None,
            })
            .await
            .unwrap();

        app.achieve_goal(&goal.id).await.unwrap();
        assert!(app
            .goal(&goal.id)
            .await
            .unwrap()
            .unwrap()
            .achievement
            .is_achieved());

        app.unachieve_goal(&goal.id).await.unwrap();
        assert_eq!(
            app.goal(&goal.id).await.unwrap().unwrap().achievement,
            Achievement::Pursuing
        );
    }

    #[tokio::test]
    async fn changing_a_habit_cadence_does_not_touch_its_other_fields() {
        let (_home, _drive, app) = ready_app().await;
        let habit = app
            .create_habit(NewHabit {
                title: "Meditation".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();

        app.set_habit_cadence(SetCadence {
            habit: &habit.id,
            cadence: Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Wed]).unwrap(),
        })
        .await
        .unwrap();

        let updated = app.habit(&habit.id).await.unwrap().unwrap();
        assert!(updated.pinned, "cadence changes must not silently unpin");
        assert!(!updated.cadence.is_due(Weekday::Tue));
    }

    #[tokio::test]
    async fn acting_on_a_missing_entity_reports_not_found() {
        let (_home, _drive, app) = ready_app().await;
        let error = app.complete_task(&TaskId::new("nope")).await.unwrap_err();
        assert!(matches!(error, AppError::NotFound { table: "task", .. }));
    }
}
