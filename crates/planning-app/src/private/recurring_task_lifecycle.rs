use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{Lifecycle, RecurringTask, RecurringTaskId};

impl PlanningApp {
    pub async fn archive_recurring_task(&self, rule: &RecurringTaskId) -> Result<(), AppError> {
        self.set_recurring_task_lifecycle(rule, Lifecycle::Archived)
            .await
    }

    pub async fn restore_recurring_task(&self, rule: &RecurringTaskId) -> Result<(), AppError> {
        self.set_recurring_task_lifecycle(rule, Lifecycle::Active)
            .await
    }

    pub async fn rename_recurring_task(
        &self,
        rule: &RecurringTaskId,
        title: String,
    ) -> Result<(), AppError> {
        self.mutate::<RecurringTask>((RecurringTaskId::TABLE, rule.to_string()), |found| {
            found.title = title;
        })
        .await?;
        Ok(())
    }

    async fn set_recurring_task_lifecycle(
        &self,
        rule: &RecurringTaskId,
        to: Lifecycle,
    ) -> Result<(), AppError> {
        self.mutate::<RecurringTask>((RecurringTaskId::TABLE, rule.to_string()), |found| {
            found.lifecycle = to;
        })
        .await?;
        Ok(())
    }
}
