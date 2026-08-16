use super::error::AppError;
use super::service::PlanningApp;
use super::views_entities::{GoalView, HabitView, ValueView};
use chrono::NaiveDate;
use planning_core::{Association, Classification, Task, TaskId};
use serde::{Deserialize, Serialize};

/// Display-only collapse of the orthogonal Completion x Lifecycle axes.
/// Archived wins because that is what the user needs to see first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    Open,
    Completed,
    Archived,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskView {
    pub id: TaskId,
    pub title: String,
    pub state: TaskState,
    pub importance: Classification,
    pub urgency: Classification,
    pub deadline: Option<NaiveDate>,
    pub overdue: bool,
    pub archived: bool,
    pub one_off: bool,
}

impl TaskView {
    /// `today` is always a home-zone date (plan 0003).
    pub fn project(task: &Task, today: NaiveDate) -> Self {
        Self {
            id: task.id.clone(),
            title: task.title.clone(),
            state: state_of(task),
            importance: task.importance,
            urgency: task.urgency,
            deadline: task.deadline,
            overdue: task.is_overdue(today),
            archived: !task.lifecycle.is_active(),
            one_off: task.one_off,
        }
    }
}

fn state_of(task: &Task) -> TaskState {
    if !task.lifecycle.is_active() {
        return TaskState::Archived;
    }
    if task.completion.is_complete() {
        return TaskState::Completed;
    }
    TaskState::Open
}

pub struct LibraryFilter {
    pub include_archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryView {
    pub values: Vec<ValueView>,
    pub goals: Vec<GoalView>,
    pub habits: Vec<HabitView>,
    pub tasks: Vec<TaskView>,
    pub associations: Vec<Association>,
}

impl PlanningApp {
    pub async fn library(&self, filter: LibraryFilter) -> Result<LibraryView, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        Ok(LibraryView {
            values: self.project_values(&filter).await?,
            goals: self.project_goals(&filter).await?,
            habits: self.project_habits(&filter).await?,
            tasks: self.project_tasks(&filter, today).await?,
            associations: self.active_associations().await?,
        })
    }

    async fn project_values(&self, filter: &LibraryFilter) -> Result<Vec<ValueView>, AppError> {
        let keep = keep_entity(filter);
        Ok(self
            .values()
            .await?
            .iter()
            .filter(|found| keep(!found.lifecycle.is_active()))
            .map(ValueView::project)
            .collect())
    }

    async fn project_goals(&self, filter: &LibraryFilter) -> Result<Vec<GoalView>, AppError> {
        let keep = keep_entity(filter);
        Ok(self
            .goals()
            .await?
            .iter()
            .filter(|found| keep(!found.lifecycle.is_active()))
            .map(GoalView::project)
            .collect())
    }

    async fn project_habits(&self, filter: &LibraryFilter) -> Result<Vec<HabitView>, AppError> {
        let keep = keep_entity(filter);
        Ok(self
            .habits()
            .await?
            .iter()
            .filter(|found| keep(!found.lifecycle.is_active()))
            .map(HabitView::project)
            .collect())
    }

    async fn project_tasks(
        &self,
        filter: &LibraryFilter,
        today: NaiveDate,
    ) -> Result<Vec<TaskView>, AppError> {
        let keep = keep_entity(filter);
        Ok(self
            .tasks()
            .await?
            .iter()
            .filter(|found| keep(!found.lifecycle.is_active()))
            .map(|found| TaskView::project(found, today))
            .collect())
    }
}

fn keep_entity(filter: &LibraryFilter) -> impl Fn(bool) -> bool + '_ {
    |archived: bool| filter.include_archived || !archived
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::entity_lifecycle::SetDeadline;
    use crate::private::test_support::ready_app;
    use chrono::Weekday;
    use planning_core::{Cadence, Classification, HabitStrength};

    #[tokio::test]
    async fn the_library_hides_archived_entries_by_default_and_can_show_them() {
        let (_home, _drive, app) = ready_app().await;
        let kept = app
            .create_task("Keep".into(), /*one_off=*/ true)
            .await
            .unwrap();
        let gone = app
            .create_task("Archive me".into(), /*one_off=*/ true)
            .await
            .unwrap();
        app.archive_task(&gone.id).await.unwrap();

        let everyday = app
            .library(LibraryFilter {
                include_archived: false,
            })
            .await
            .unwrap();
        assert_eq!(everyday.tasks.len(), 1);
        assert_eq!(everyday.tasks[0].id, kept.id);

        let with_archive = app
            .library(LibraryFilter {
                include_archived: true,
            })
            .await
            .unwrap();
        assert_eq!(with_archive.tasks.len(), 2);
        let archived = with_archive
            .tasks
            .iter()
            .find(|view| view.id == gone.id)
            .unwrap();
        assert!(
            archived.archived,
            "archived entries are shown honestly, not hidden"
        );
        assert_eq!(archived.state, TaskState::Archived);
    }

    #[tokio::test]
    async fn overdue_is_projected_from_the_home_date_not_stored() {
        let (_home, _drive, app) = ready_app().await;
        let task = app
            .create_task("File taxes".into(), /*one_off=*/ true)
            .await
            .unwrap();
        app.set_task_deadline(SetDeadline {
            task: &task.id,
            deadline: Some(NaiveDate::from_ymd_opt(2026, 8, 6).unwrap()),
        })
        .await
        .unwrap();

        let library = app
            .library(LibraryFilter {
                include_archived: false,
            })
            .await
            .unwrap();
        assert!(library.tasks[0].overdue);

        app.complete_task(&task.id).await.unwrap();
        let after = app
            .library(LibraryFilter {
                include_archived: false,
            })
            .await
            .unwrap();
        assert!(!after.tasks[0].overdue);
        assert_eq!(
            after.tasks[0].deadline,
            Some(NaiveDate::from_ymd_opt(2026, 8, 6).unwrap())
        );
    }

    #[test]
    fn task_views_serialize_exactly_as_the_frontend_types_declare() {
        let view = TaskView {
            id: TaskId::new("t1"),
            title: "File taxes".into(),
            state: TaskState::Open,
            importance: Classification::High,
            urgency: Classification::Unclassified,
            deadline: Some(NaiveDate::from_ymd_opt(2026, 8, 6).unwrap()),
            overdue: true,
            archived: false,
            one_off: true,
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"id":"t1","title":"File taxes","state":"open","importance":"high","urgency":"unclassified","deadline":"2026-08-06","overdue":true,"archived":false,"oneOff":true}"#
        );
    }

    #[test]
    fn goal_views_serialize_exactly_as_the_frontend_types_declare() {
        let view = GoalView {
            id: planning_core::GoalId::new("g1"),
            title: "Career".into(),
            achieved: true,
            target_date: Some(NaiveDate::from_ymd_opt(2026, 12, 1).unwrap()),
            archived: false,
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"id":"g1","title":"Career","achieved":true,"targetDate":"2026-12-01","archived":false}"#
        );
    }

    #[test]
    fn habit_views_serialize_exactly_as_the_frontend_types_declare() {
        let view = HabitView {
            id: planning_core::HabitId::new("h1"),
            title: "Writing".into(),
            cadence: Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Wed]).unwrap(),
            strength: HabitStrength::Established,
            pinned: true,
            archived: false,
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"id":"h1","title":"Writing","cadence":{"kind":"onWeekdays","days":["mon","wed"]},"strength":"established","pinned":true,"archived":false}"#
        );
    }

    #[test]
    fn value_views_serialize_exactly_as_the_frontend_types_declare() {
        let view = ValueView {
            id: planning_core::ValueId::new("v1"),
            title: "Integrity".into(),
            archived: false,
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"id":"v1","title":"Integrity","archived":false}"#
        );
    }

    #[test]
    fn library_views_serialize_associations_for_the_frontend() {
        let view = LibraryView {
            values: vec![],
            goals: vec![],
            habits: vec![],
            tasks: vec![],
            associations: vec![],
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"values":[],"goals":[],"habits":[],"tasks":[],"associations":[]}"#
        );
    }
}
