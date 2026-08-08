use super::calendar_week::CalendarWeek;
use super::clock::Clock;
use super::ids::{TaskId, WeeklyFocusId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct StartFocus<'a> {
    pub week: CalendarWeek,
    pub clock: &'a dyn Clock,
}

/// An ordered selection of tasks for one ISO calendar week. Stores ids only —
/// archived marking is projected at read time (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyFocus {
    pub id: WeeklyFocusId,
    pub week: CalendarWeek,
    pub tasks: Vec<TaskId>,
    pub created_at: DateTime<Utc>,
}

impl WeeklyFocus {
    /// The record key IS the week label, so "one focus per week" is a property
    /// of the store rather than something every caller must remember to check.
    pub fn key(week: CalendarWeek) -> String {
        week.label()
    }

    pub fn start(request: StartFocus<'_>) -> Self {
        Self {
            id: WeeklyFocusId::new(Self::key(request.week)),
            week: request.week,
            tasks: Vec::new(),
            created_at: request.clock.now(),
        }
    }

    pub fn add(&mut self, task: TaskId) -> bool {
        if self.tasks.contains(&task) {
            return false;
        }
        self.tasks.push(task);
        true
    }

    pub fn remove(&mut self, task: &TaskId) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|found| found != task);
        self.tasks.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::calendar_week::CalendarWeek;
    use crate::private::clock::FixedClock;
    use crate::private::daily_plan::{DailyPlan, StartPlan};
    use chrono::{NaiveDate, TimeZone, Utc};

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    fn week() -> CalendarWeek {
        CalendarWeek::containing(day())
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn focus() -> WeeklyFocus {
        WeeklyFocus::start(StartFocus {
            week: week(),
            clock: &clock(),
        })
    }

    #[test]
    fn the_key_is_the_week_label_so_a_week_can_only_have_one_focus() {
        assert_eq!(WeeklyFocus::key(week()), "2026-W32");
    }

    #[test]
    fn adding_the_same_task_twice_is_idempotent() {
        let mut focus = focus();
        assert!(focus.add(TaskId::new("t1")));
        assert!(!focus.add(TaskId::new("t1")));
        assert_eq!(focus.tasks, vec![TaskId::new("t1")]);
    }

    #[test]
    fn removing_an_absent_task_returns_false() {
        let mut focus = focus();
        assert!(!focus.remove(&TaskId::new("t1")));
    }

    #[test]
    fn a_task_in_a_focus_is_unaffected_by_being_added_to_a_daily_plan() {
        let mut focus = focus();
        let task = TaskId::new("t1");
        focus.add(task.clone());

        let mut plan = DailyPlan::start(StartPlan {
            date: day(),
            habits: vec![],
            clock: &clock(),
        });
        plan.select(task.clone());

        assert_eq!(focus.tasks, vec![task]);
    }
}
