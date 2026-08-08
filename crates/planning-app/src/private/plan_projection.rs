use super::check_in_use_cases::DateRange;
use super::error::AppError;
use super::plan_views::PlanHabitView;
use super::plan_views::PlanTaskView;
use super::service::PlanningApp;
use planning_core::{DailyPlan, HabitId, TaskId};
use std::collections::HashMap;

impl PlanningApp {
    pub(crate) async fn project_plan_tasks(
        &self,
        plan: &DailyPlan,
    ) -> Result<Vec<PlanTaskView>, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let tasks: HashMap<TaskId, _> = self
            .tasks()
            .await?
            .into_iter()
            .map(|task| (task.id.clone(), task))
            .collect();
        Ok(plan
            .tasks
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
                tasks
                    .get(id)
                    .map(|task| PlanTaskView::project(task, today, index as u32))
            })
            .collect())
    }

    pub(crate) async fn project_plan_habits(
        &self,
        plan: &DailyPlan,
    ) -> Result<Vec<PlanHabitView>, AppError> {
        let habits: HashMap<HabitId, _> = self
            .habits()
            .await?
            .into_iter()
            .map(|habit| (habit.id.clone(), habit))
            .collect();
        let check_ins = self
            .check_ins_between(DateRange {
                from: plan.date,
                to: plan.date,
            })
            .await?;
        let outcomes: HashMap<_, _> = check_ins
            .into_iter()
            .map(|record| (record.habit, record.outcome))
            .collect();
        Ok(plan
            .habits
            .iter()
            .filter_map(|id| {
                habits.get(id).map(|habit| PlanHabitView {
                    id: habit.id.clone(),
                    title: habit.title.clone(),
                    cadence: habit.cadence,
                    archived: !habit.lifecycle.is_active(),
                    unpinned: !habit.pinned,
                    outcome: outcomes.get(id).copied(),
                })
            })
            .collect())
    }
}
