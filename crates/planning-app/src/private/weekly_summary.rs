use super::check_in_use_cases::DateRange;
use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{
    Achievement, CalendarWeek, CheckInOutcome, Completion, Goal, Habit, HabitId, Task,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitSummary {
    pub title: String,
    pub done: u32,
    pub skipped: u32,
    pub not_completed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummary {
    pub week: CalendarWeek,
    pub completed: Vec<String>,
    pub still_open: usize,
    pub overdue: Vec<String>,
    pub habits: Vec<HabitSummary>,
    pub goals_achieved: Vec<String>,
}

impl PlanningApp {
    /// Computed fresh on every call. Nothing here is stored, which is why a
    /// correction made weeks later appears in an old report (ADR 0002).
    pub async fn weekly_summary(&self, week: CalendarWeek) -> Result<WeeklySummary, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let tasks = self.tasks().await?;

        Ok(WeeklySummary {
            week,
            completed: completed_in_week(&tasks, week),
            still_open: still_open_count(&tasks),
            overdue: overdue_tasks(&tasks, today),
            habits: self.summarize_habits(week).await?,
            goals_achieved: self.goals_achieved_in(week).await?,
        })
    }

    async fn summarize_habits(&self, week: CalendarWeek) -> Result<Vec<HabitSummary>, AppError> {
        let check_ins = self
            .check_ins_between(DateRange {
                from: week.monday(),
                to: week.sunday(),
            })
            .await?;
        let mut tallies: HashMap<HabitId, (u32, u32, u32)> = HashMap::new();
        for check_in in check_ins {
            let entry = tallies.entry(check_in.habit).or_default();
            match check_in.outcome {
                CheckInOutcome::Done => entry.0 += 1,
                CheckInOutcome::Skipped => entry.1 += 1,
                CheckInOutcome::NotCompleted => entry.2 += 1,
            }
        }
        Ok(self
            .habits()
            .await?
            .into_iter()
            .filter_map(|habit| {
                tallies
                    .get(&habit.id)
                    .map(|counts| summarize(&habit, *counts))
            })
            .collect())
    }

    async fn goals_achieved_in(&self, week: CalendarWeek) -> Result<Vec<String>, AppError> {
        Ok(self
            .goals()
            .await?
            .into_iter()
            .filter_map(|goal| goal_achieved_in_week(&goal, week))
            .collect())
    }
}

fn completed_in_week(tasks: &[Task], week: CalendarWeek) -> Vec<String> {
    tasks
        .iter()
        .filter(|task| matches!(task.completion, Completion::Completed { on } if week.contains(on)))
        .map(|task| task.title.clone())
        .collect()
}

fn overdue_tasks(tasks: &[Task], today: chrono::NaiveDate) -> Vec<String> {
    tasks
        .iter()
        .filter(|task| task.is_overdue(today))
        .map(|task| task.title.clone())
        .collect()
}

fn still_open_count(tasks: &[Task]) -> usize {
    tasks
        .iter()
        .filter(|task| task.lifecycle.is_active() && !task.completion.is_complete())
        .count()
}

fn goal_achieved_in_week(goal: &Goal, week: CalendarWeek) -> Option<String> {
    let Achievement::Achieved { on } = goal.achievement else {
        return None;
    };
    if !week.contains(on) {
        return None;
    }
    Some(goal.title.clone())
}

fn summarize(habit: &Habit, counts: (u32, u32, u32)) -> HabitSummary {
    HabitSummary {
        title: habit.title.clone(),
        done: counts.0,
        skipped: counts.1,
        not_completed: counts.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::entity_lifecycle::SetDeadline;
    use crate::private::library::NewHabit;
    use crate::private::summary_markdown;
    use crate::private::test_support::ready_app_at;
    use chrono::{Duration, TimeZone, Utc};
    use planning_core::{Cadence, FixedClock};
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn the_summary_reflects_current_data_including_later_corrections() {
        let (_home, _drive, app, clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let today = app.calendar().unwrap().today(app.clock_ref());

        let done = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.complete_task(&done.id).await.unwrap();
        let open = app.create_task("Call the bank".into()).await.unwrap();
        app.set_task_deadline(SetDeadline {
            task: &open.id,
            deadline: Some(today - Duration::days(1)),
        })
        .await
        .unwrap();

        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        app.record_check_in(crate::private::check_in_use_cases::CheckInRequest {
            habit: habit.id.clone(),
            date: today,
            outcome: CheckInOutcome::NotCompleted,
        })
        .await
        .unwrap();

        let before = app.weekly_summary(week).await.unwrap();
        assert_eq!(before.completed, vec!["Prepare portfolio".to_string()]);
        assert_eq!(before.overdue, vec!["Call the bank".to_string()]);
        assert_eq!(before.habits[0].not_completed, 1);

        clock.advance(Duration::days(14));
        app.record_check_in(crate::private::check_in_use_cases::CheckInRequest {
            habit: habit.id.clone(),
            date: today,
            outcome: CheckInOutcome::Done,
        })
        .await
        .unwrap();

        let after = app.weekly_summary(week).await.unwrap();
        assert_eq!(
            after.habits[0].done, 1,
            "summaries are never frozen (ADR 0002)"
        );
        assert_eq!(after.habits[0].not_completed, 0);
    }

    #[tokio::test]
    async fn the_summary_counts_only_the_weeks_own_days() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        let today = app.calendar().unwrap().today(app.clock_ref());

        app.record_check_in(crate::private::check_in_use_cases::CheckInRequest {
            habit: habit.id.clone(),
            date: today,
            outcome: CheckInOutcome::Done,
        })
        .await
        .unwrap();
        app.record_check_in(crate::private::check_in_use_cases::CheckInRequest {
            habit: habit.id.clone(),
            date: week.next().monday(),
            outcome: CheckInOutcome::Done,
        })
        .await
        .unwrap();

        assert_eq!(app.weekly_summary(week).await.unwrap().habits[0].done, 1);
        assert_eq!(
            app.weekly_summary(week.next()).await.unwrap().habits[0].done,
            1
        );
    }

    #[test]
    fn the_rendered_summary_never_scores_or_gamifies() {
        let summary = WeeklySummary {
            week: CalendarWeek::containing(chrono::NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()),
            completed: vec!["Prepare portfolio".into()],
            still_open: 3,
            overdue: vec!["Call the bank".into()],
            habits: vec![HabitSummary {
                title: "Writing".into(),
                done: 4,
                skipped: 1,
                not_completed: 2,
            }],
            goals_achieved: vec![],
        };

        let markdown = summary_markdown::render(&summary);
        assert!(markdown.contains("Prepare portfolio"));
        assert!(markdown.contains("Writing"));
        for banned in ["%", "streak", "Streak", "score", "Score", "🔥"] {
            assert!(
                !markdown.contains(banned),
                "PRODUCT.md forbids {banned} in reports"
            );
        }
    }

    #[test]
    fn weekly_summaries_serialize_exactly_as_the_frontend_types_declare() {
        let summary = WeeklySummary {
            week: CalendarWeek::containing(chrono::NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()),
            completed: vec!["Prepare portfolio".into()],
            still_open: 3,
            overdue: vec!["Call the bank".into()],
            habits: vec![HabitSummary {
                title: "Writing".into(),
                done: 4,
                skipped: 1,
                not_completed: 2,
            }],
            goals_achieved: vec!["Career transition".into()],
        };
        assert_eq!(
            serde_json::to_string(&summary).unwrap(),
            r#"{"week":"2026-W32","completed":["Prepare portfolio"],"stillOpen":3,"overdue":["Call the bank"],"habits":[{"title":"Writing","done":4,"skipped":1,"notCompleted":2}],"goalsAchieved":["Career transition"]}"#
        );
    }
}
