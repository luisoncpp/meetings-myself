use super::clock::Clock;
use super::ids::{DailyPlanId, HabitId, TaskId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct StartPlan<'a> {
    pub date: NaiveDate,
    pub habits: Vec<HabitId>,
    pub clock: &'a dyn Clock,
}

/// A fresh, editable ordered selection for one calendar day. Stores ids only —
/// archived and unpinned marking is projected at read time, which is what makes
/// forward-only propagation free (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPlan {
    pub id: DailyPlanId,
    pub date: NaiveDate,
    pub tasks: Vec<TaskId>,
    pub habits: Vec<HabitId>,
    pub created_at: DateTime<Utc>,
}

impl DailyPlan {
    /// The record key IS the date, so "one plan per day" is a property of the
    /// store rather than something every caller must remember to check.
    pub fn key(date: NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    pub fn start(request: StartPlan<'_>) -> Self {
        Self {
            id: DailyPlanId::new(Self::key(request.date)),
            date: request.date,
            tasks: Vec::new(),
            habits: request.habits,
            created_at: request.clock.now(),
        }
    }

    pub fn select(&mut self, task: TaskId) -> bool {
        if self.tasks.contains(&task) {
            return false;
        }
        self.tasks.push(task);
        true
    }

    /// Removing from the plan never touches the Task Pool (CONTEXT.md).
    pub fn unselect(&mut self, task: &TaskId) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|found| found != task);
        self.tasks.len() != before
    }

    /// Rejects anything that is not a permutation, so a drag-and-drop bug cannot
    /// silently drop or invent entries.
    pub fn reorder(&mut self, order: Vec<TaskId>) -> bool {
        if order.len() != self.tasks.len() {
            return false;
        }
        let proposed: HashSet<&TaskId> = order.iter().collect();
        let current: HashSet<&TaskId> = self.tasks.iter().collect();
        if proposed != current {
            return false;
        }
        self.tasks = order;
        true
    }

    pub fn include_habit(&mut self, habit: HabitId) -> bool {
        if self.habits.contains(&habit) {
            return false;
        }
        self.habits.push(habit);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn plan() -> DailyPlan {
        DailyPlan::start(StartPlan {
            date: day(),
            habits: vec![],
            clock: &clock(),
        })
    }

    #[test]
    fn the_key_is_the_date_so_a_day_can_only_have_one_plan() {
        assert_eq!(DailyPlan::key(day()), "2026-08-07");
    }

    #[test]
    fn selecting_the_same_task_twice_does_not_duplicate_it() {
        let mut plan = plan();
        assert!(plan.select(TaskId::new("t1")));
        assert!(!plan.select(TaskId::new("t1")), "second selection is a no-op");
        assert_eq!(plan.tasks, vec![TaskId::new("t1")]);
    }

    #[test]
    fn selection_is_reversible_and_order_is_preserved() {
        let mut plan = plan();
        for id in ["t1", "t2", "t3"] {
            plan.select(TaskId::new(id));
        }
        assert!(plan.unselect(&TaskId::new("t2")));
        assert_eq!(plan.tasks, vec![TaskId::new("t1"), TaskId::new("t3")]);
        assert!(!plan.unselect(&TaskId::new("t2")), "removing twice is a no-op");
    }

    #[test]
    fn reorder_accepts_only_a_permutation_of_the_current_tasks() {
        let mut plan = plan();
        for id in ["t1", "t2", "t3"] {
            plan.select(TaskId::new(id));
        }

        assert!(plan.reorder(vec![
            TaskId::new("t3"),
            TaskId::new("t1"),
            TaskId::new("t2"),
        ]));
        assert_eq!(
            plan.tasks,
            vec![
                TaskId::new("t3"),
                TaskId::new("t1"),
                TaskId::new("t2"),
            ]
        );

        // A drag-and-drop bug must not be able to drop or invent entries.
        assert!(!plan.reorder(vec![TaskId::new("t3"), TaskId::new("t1")]));
        assert!(!plan.reorder(vec![
            TaskId::new("t3"),
            TaskId::new("t1"),
            TaskId::new("t9"),
        ]));
        assert_eq!(plan.tasks.len(), 3, "a rejected reorder changes nothing");
    }
}
