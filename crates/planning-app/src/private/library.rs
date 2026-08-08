use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{
    Cadence, CreateGoal, CreateHabit, CreateTask, CreateValue, Goal, GoalId, Habit, HabitId, Task,
    TaskId, Value, ValueId,
};

pub struct NewGoal {
    pub title: String,
    pub target_date: Option<NaiveDate>,
}

pub struct NewHabit {
    pub title: String,
    pub cadence: Cadence,
}

impl PlanningApp {
    pub async fn create_value(&self, title: String) -> Result<Value, AppError> {
        let value = Value::create(CreateValue {
            title,
            clock: self.clock.as_ref(),
        })?;
        self.store(ValueId::TABLE, value.id.as_str(), &value)
            .await?;
        Ok(value)
    }

    pub async fn create_goal(&self, request: NewGoal) -> Result<Goal, AppError> {
        let goal = Goal::create(CreateGoal {
            title: request.title,
            target_date: request.target_date,
            clock: self.clock.as_ref(),
        })?;
        self.store(GoalId::TABLE, goal.id.as_str(), &goal).await?;
        Ok(goal)
    }

    pub async fn create_task(&self, title: String) -> Result<Task, AppError> {
        let task = Task::create(CreateTask {
            title,
            clock: self.clock.as_ref(),
        })?;
        self.store(TaskId::TABLE, task.id.as_str(), &task).await?;
        Ok(task)
    }

    pub async fn create_habit(&self, request: NewHabit) -> Result<Habit, AppError> {
        let habit = Habit::create(CreateHabit {
            title: request.title,
            cadence: request.cadence,
            clock: self.clock.as_ref(),
        })?;
        self.store(HabitId::TABLE, habit.id.as_str(), &habit)
            .await?;
        Ok(habit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app;
    use chrono::Weekday;
    use planning_core::{Classification, Lifecycle};

    #[tokio::test]
    async fn a_task_created_from_a_title_is_persisted_with_defaults() {
        let (_home, _drive, app) = ready_app().await;

        let task = app.create_task("Draft the letter".into()).await.unwrap();
        assert_eq!(task.importance, Classification::Unclassified);
        assert_eq!(task.lifecycle, Lifecycle::Active);

        let stored = app.tasks().await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, task.id);
    }

    #[tokio::test]
    async fn a_habit_requires_a_cadence_and_arrives_pinned() {
        let (_home, _drive, app) = ready_app().await;
        let habit = app
            .create_habit(NewHabit {
                title: "Writing practice".into(),
                cadence: Cadence::new_on_weekdays(&[Weekday::Mon]).unwrap(),
            })
            .await
            .unwrap();
        assert!(habit.pinned);
    }

    #[tokio::test]
    async fn creation_is_refused_before_setup_completes() {
        use crate::private::service::StartRequest;
        use chrono::TimeZone;
        use planning_core::FixedClock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let home = TempDir::new().unwrap();
        let app = PlanningApp::start(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(
                chrono::Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
            )),
        })
        .await
        .unwrap();

        let error = app.create_task("Anything".into()).await.unwrap_err();
        assert!(matches!(
            error,
            AppError::NotReady(_) | AppError::NoDatabase
        ));
    }

    #[tokio::test]
    async fn a_blank_title_is_a_domain_error_not_a_panic() {
        let (_home, _drive, app) = ready_app().await;
        assert!(matches!(
            app.create_task("   ".into()).await.unwrap_err(),
            AppError::Domain(_)
        ));
    }
}
