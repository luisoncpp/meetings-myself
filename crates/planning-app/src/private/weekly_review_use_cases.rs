use super::error::AppError;
use super::service::PlanningApp;
use super::summary_markdown;
use super::views::TaskView;
use super::weekly_summary::WeeklySummary;
use planning_core::{CalendarWeek, StartReview, WeeklyReview, WeeklyReviewId};
use planning_reports::{ReportFrontMatter, SaveBody, WriteReport};
use serde::Serialize;
use std::path::PathBuf;

pub struct SaveReflection {
    pub week: CalendarWeek,
    pub reflection: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReviewView {
    pub week: CalendarWeek,
    pub summary: WeeklySummary,
    pub reflection: String,
    pub previous_report: Option<String>,
    pub next_week_focus: Vec<TaskView>,
    pub report_path: PathBuf,
}

impl PlanningApp {
    pub async fn open_current_review(&self) -> Result<WeeklyReviewView, AppError> {
        let week = self.calendar()?.current_week(self.clock.as_ref());
        self.open_weekly_review(week).await
    }

    /// Idempotent by construction: the review record's key is the week label and
    /// the report's filename is derived from it, so reopening refreshes rather
    /// than duplicating (A4).
    pub async fn open_weekly_review(
        &self,
        week: CalendarWeek,
    ) -> Result<WeeklyReviewView, AppError> {
        self.touch_review(week).await?;
        let summary = self.weekly_summary(week).await?;
        self.regenerate_report(&summary).await?;
        self.weekly_focus(week.next()).await?;
        self.build_review_view(week, summary).await
    }

    pub async fn save_reflection(&self, request: SaveReflection) -> Result<(), AppError> {
        let summary = self.weekly_summary(request.week).await?;
        self.regenerate_report(&summary).await?;
        self.require_reports()?.save_reflection(SaveBody {
            week_label: request.week.label(),
            reflection: request.reflection,
        })?;
        Ok(())
    }

    pub fn report_path(&self, week: CalendarWeek) -> Result<PathBuf, AppError> {
        Ok(self.require_reports()?.path_for(&week.label()))
    }

    async fn build_review_view(
        &self,
        week: CalendarWeek,
        summary: WeeklySummary,
    ) -> Result<WeeklyReviewView, AppError> {
        let reports = self.require_reports()?;
        let document = reports.read(&week.label())?;
        Ok(WeeklyReviewView {
            week,
            summary,
            reflection: document
                .map(|found| planning_reports::SummaryBlock::reflection(&found.body))
                .unwrap_or_default(),
            previous_report: reports
                .read(&week.previous().label())?
                .map(|found| found.body),
            next_week_focus: self.focus_task_views(week.next()).await?,
            report_path: reports.path_for(&week.label()),
        })
    }

    async fn regenerate_report(&self, summary: &WeeklySummary) -> Result<(), AppError> {
        let week = summary.week;
        self.require_reports()?.write(WriteReport {
            front_matter: ReportFrontMatter {
                week: week.label(),
                week_start: week.monday(),
                week_end: week.sunday(),
                schema: ReportFrontMatter::SCHEMA,
                generated_at: self.clock.now(),
            },
            summary_markdown: summary_markdown::render(summary),
        })?;
        Ok(())
    }

    async fn touch_review(&self, week: CalendarWeek) -> Result<(), AppError> {
        let key = WeeklyReview::key(week);
        let existing: Option<WeeklyReview> = self.load_one(WeeklyReviewId::TABLE, &key).await?;
        let mut review = existing.unwrap_or_else(|| {
            WeeklyReview::start(StartReview {
                week,
                clock: self.clock.as_ref(),
            })
        });
        review.last_opened_at = self.clock.now();
        self.store(WeeklyReviewId::TABLE, &key, &review).await?;
        Ok(())
    }

    async fn focus_task_views(&self, week: CalendarWeek) -> Result<Vec<TaskView>, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let focus = self.weekly_focus(week).await?;
        let mut views = Vec::new();
        for task_id in &focus.tasks {
            let Some(task) = self.task(task_id).await? else {
                continue;
            };
            views.push(TaskView::project(&task, today));
        }
        Ok(views)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::library::NewGoal;
    use crate::private::test_support::ready_app_at;
    use crate::private::weekly_focus_use_cases::FocusChange;
    use chrono::{Duration, TimeZone, Utc};
    use planning_core::FixedClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn opening_a_review_creates_one_report_prepares_next_week_and_shows_the_prior_one() {
        let (_home, drive, app, clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());

        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.complete_task(&task.id).await.unwrap();

        let view = app.open_weekly_review(week).await.unwrap();
        assert_eq!(view.week, week);
        assert!(view
            .summary
            .completed
            .contains(&"Prepare portfolio".to_string()));
        assert_eq!(view.previous_report, None, "there is no earlier week yet");
        assert!(view.report_path.exists());

        let next = app.weekly_focus(week.next()).await.unwrap();
        assert!(next.tasks.is_empty());

        clock.advance(Duration::days(7));
        let later = app.calendar().unwrap().current_week(app.clock_ref());
        let second = app.open_weekly_review(later).await.unwrap();
        assert!(
            second
                .previous_report
                .unwrap()
                .contains("Prepare portfolio"),
            "A4: the review shows the prior report"
        );

        let files: Vec<_> = std::fs::read_dir(drive.path().join("weekly-reports"))
            .unwrap()
            .collect();
        assert_eq!(files.len(), 2, "one file per week, never more");
    }

    #[tokio::test]
    async fn reopening_a_past_review_refreshes_it_without_creating_a_duplicate() {
        let (_home, drive, app, clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());

        app.open_weekly_review(week).await.unwrap();
        app.save_reflection(SaveReflection {
            week,
            reflection: "## Reflection\n\nA quiet week.\n".into(),
        })
        .await
        .unwrap();

        clock.advance(Duration::days(21));
        let task = app.create_task("Late entry".into()).await.unwrap();
        app.complete_task(&task.id).await.unwrap();

        let reopened = app.open_weekly_review(week).await.unwrap();
        assert!(
            reopened.reflection.contains("A quiet week."),
            "reflection is preserved as written"
        );
        assert!(
            !reopened
                .summary
                .completed
                .contains(&"Late entry".to_string()),
            "the Task was completed in a later week, so it belongs to that week's summary"
        );

        let files: Vec<_> = std::fs::read_dir(drive.path().join("weekly-reports"))
            .unwrap()
            .collect();
        assert_eq!(
            files.len(),
            1,
            "A4: reopening never creates a second report"
        );
    }

    #[tokio::test]
    async fn every_review_action_is_also_available_without_a_review() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let week = app.calendar().unwrap().current_week(app.clock_ref());
        let goal = app
            .create_goal(NewGoal {
                title: "Career transition".into(),
                target_date: None,
            })
            .await
            .unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();

        app.achieve_goal(&goal.id).await.unwrap();
        app.add_to_focus(FocusChange {
            week: week.next(),
            task: task.id,
        })
        .await
        .unwrap();

        assert!(app
            .goal(&goal.id)
            .await
            .unwrap()
            .unwrap()
            .achievement
            .is_achieved());
        assert_eq!(app.weekly_focus(week.next()).await.unwrap().tasks.len(), 1);
    }

    #[test]
    fn weekly_review_views_serialize_exactly_as_the_frontend_types_declare() {
        use super::super::views::{TaskState, TaskView};
        use planning_core::{Classification, TaskId};

        let week = CalendarWeek::containing(chrono::NaiveDate::from_ymd_opt(2026, 8, 7).unwrap());
        let view = WeeklyReviewView {
            week,
            summary: super::super::weekly_summary::WeeklySummary {
                week,
                completed: vec!["Prepare portfolio".into()],
                still_open: 1,
                overdue: vec![],
                habits: vec![super::super::weekly_summary::HabitSummary {
                    title: "Writing".into(),
                    done: 2,
                    skipped: 0,
                    not_completed: 1,
                }],
                goals_achieved: vec![],
            },
            reflection: "A quiet week.".into(),
            previous_report: Some("Prior week done.".into()),
            next_week_focus: vec![TaskView {
                id: TaskId::new("t1"),
                title: "Call the bank".into(),
                state: TaskState::Open,
                importance: Classification::High,
                urgency: Classification::Low,
                deadline: None,
                overdue: false,
                archived: false,
            }],
            report_path: std::path::PathBuf::from("/sync/weekly-reports/2026-W32-weekly-report.md"),
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"week":"2026-W32","summary":{"week":"2026-W32","completed":["Prepare portfolio"],"stillOpen":1,"overdue":[],"habits":[{"title":"Writing","done":2,"skipped":0,"notCompleted":1}],"goalsAchieved":[]},"reflection":"A quiet week.","previousReport":"Prior week done.","nextWeekFocus":[{"id":"t1","title":"Call the bank","state":"open","importance":"high","urgency":"low","deadline":null,"overdue":false,"archived":false}],"reportPath":"/sync/weekly-reports/2026-W32-weekly-report.md"}"#
        );
    }
}
