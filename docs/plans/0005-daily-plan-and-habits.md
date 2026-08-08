# Daily Plans, Habits & Recurrence — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](0001-self-planning-app.md) first. Requires
> [0004-planning-domain.md](done/0004-planning-domain.md) to be complete.

**Goal:** Daily Plans with ordered task selection and habit check-ins, a Weekly Focus that guides
without scheduling, and duplicate-safe Recurring Task materialization — all obeying forward-only
propagation.

**Architecture:** Three deliberate uses of the record key as an invariant, so uniqueness is
structural rather than something a query has to remember:

| Record | Key | Invariant it enforces |
|--------|-----|-----------------------|
| `DailyPlan` | the ISO date, `"2026-08-07"` | exactly one plan per day |
| `WeeklyFocus` | the week label, `"2026-W32"` | exactly one focus per Calendar Week |
| `HabitCheckIn` | `"{habitId}:{date}"` | one check-in per Habit per day; correcting is an upsert |
| `Occurrence` | `"{ruleId}:{date}"` | a Recurring Task materializes at most once per date |

A Daily Plan stores **ids only**. Archived and unpinned marking is projected at read time by
resolving each id against current entity state — which is what makes forward-only propagation cost
nothing. No archive operation ever writes to a plan.

**Tech Stack:** As plan 0004. No new dependencies.

---

## Global constraints

See [0001-self-planning-app.md](0001-self-planning-app.md#global-constraints). Load-bearing here:

- **Forward-only.** Archiving, unpinning, and cadence changes never rewrite an existing Daily Plan
  or Weekly Focus. Affected entries stay in place, marked, and still completable.
- **Recurring Task rules are factories.** Editing or archiving a rule affects future occurrences
  only; a materialized occurrence is an ordinary Task.
- **Outcomes stay correctable.** Any past Habit Check-in can be changed at any time. Completing a
  Task is never gated on it being in a Daily Plan.
- **Selecting a Task into a plan or focus never removes it from the Task Pool.**
- **Task selection is manual.** The Weekly Focus orders the Task Pool; it never auto-fills a plan.
  Habits are the only entries added automatically, and only when pinned and due.

---

## File structure

| File | Responsibility |
|------|----------------|
| `crates/planning-core/src/private/ids.rs` | *(modify)* add `OccurrenceId` |
| `crates/planning-core/src/private/daily_plan.rs` | `DailyPlan` |
| `crates/planning-core/src/private/weekly_focus.rs` | `WeeklyFocus` |
| `crates/planning-core/src/private/check_in.rs` | `HabitCheckIn`, `CheckInOutcome` |
| `crates/planning-core/src/private/recurrence.rs` | `Recurrence` |
| `crates/planning-core/src/private/recurring_task.rs` | `RecurringTask`, `Occurrence` |
| `crates/planning-app/src/private/daily_plan_use_cases.rs` | Open, select, reorder, remove |
| `crates/planning-app/src/private/check_in_use_cases.rs` | Record and correct check-ins |
| `crates/planning-app/src/private/weekly_focus_use_cases.rs` | Focus editing |
| `crates/planning-app/src/private/materialization.rs` | Idempotent occurrence generation |
| `crates/planning-app/src/private/plan_views.rs` | `DailyPlanView`, `TaskPoolView` |
| `src-tauri/src/private/plan_commands.rs` | Tauri commands |
| `src/lib/domain/index.ts` | *(modify)* mirror types |
| `src/lib/api/index.ts` | *(modify)* plan API |
| `docs/architecture/daily-planning.md` | New architecture doc |
| `docs/flows/opening-todays-plan.md` | New flow doc |
| `docs/flows/archiving-a-habit-already-in-a-plan.md` | New flow doc |

---

### Task 1: `DailyPlan`, `WeeklyFocus`, and `HabitCheckIn` entities

**Files:**
- Create: `crates/planning-core/src/private/daily_plan.rs`, `weekly_focus.rs`, `check_in.rs`
- Modify: `crates/planning-core/src/private/ids.rs` (add `OccurrenceId => "occurrence"`),
  `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: ids, `Clock`, `CalendarWeek`.
- Produces:

```rust
pub struct DailyPlan { pub id: DailyPlanId, pub date: NaiveDate,
                       pub tasks: Vec<TaskId>, pub habits: Vec<HabitId>,
                       pub created_at: DateTime<Utc> }
impl DailyPlan {
    pub fn key(date: NaiveDate) -> String;            // "2026-08-07"
    pub fn start(StartPlan { date, habits, clock }) -> Self;
    pub fn select(&mut self, TaskId) -> bool;         // false if already present
    pub fn unselect(&mut self, &TaskId) -> bool;
    pub fn reorder(&mut self, Vec<TaskId>) -> bool;   // false if not a permutation
    pub fn include_habit(&mut self, HabitId) -> bool;
}

pub struct WeeklyFocus { pub id: WeeklyFocusId, pub week: CalendarWeek,
                         pub tasks: Vec<TaskId>, pub created_at: DateTime<Utc> }
impl WeeklyFocus {
    pub fn key(week: CalendarWeek) -> String;         // "2026-W32"
    pub fn start(StartFocus { week, clock }) -> Self;
    pub fn add(&mut self, TaskId) -> bool;
    pub fn remove(&mut self, &TaskId) -> bool;
}

pub enum CheckInOutcome { Done, Skipped, NotCompleted }
pub struct HabitCheckIn { pub id: HabitCheckInId, pub habit: HabitId, pub date: NaiveDate,
                          pub outcome: CheckInOutcome, pub recorded_at: DateTime<Utc> }
impl HabitCheckIn {
    pub fn key(habit: &HabitId, date: NaiveDate) -> String;   // "abc:2026-08-07"
    pub fn record(RecordCheckIn { habit, date, outcome, clock }) -> Self;
}
```

`RecordCheckIn` has four fields, which is fine — the 3-parameter rule is about *parameters*, and
this is one struct parameter. That is exactly why these request structs exist.

- [ ] **Step 1: Write the failing `DailyPlan` test**

```rust
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
        DailyPlan::start(StartPlan { date: day(), habits: vec![], clock: &clock() })
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

        assert!(plan.reorder(vec![TaskId::new("t3"), TaskId::new("t1"), TaskId::new("t2")]));
        assert_eq!(plan.tasks, vec![TaskId::new("t3"), TaskId::new("t1"), TaskId::new("t2")]);

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
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p planning-core daily_plan
```

Expected: FAIL — `cannot find struct 'DailyPlan'`.

- [ ] **Step 3: Implement `daily_plan.rs`**

```rust
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
```

- [ ] **Step 4: Write and pass the `WeeklyFocus` and `HabitCheckIn` tests**

`weekly_focus.rs` tests: the key is the week label (`"2026-W32"`); `add` is idempotent; `remove`
returns `false` when absent; a Task in a focus is unaffected by being added to a Daily Plan.

`check_in.rs` tests:

```rust
#[test]
fn the_key_pairs_habit_and_date_so_one_day_holds_one_outcome() {
    let habit = HabitId::new("h1");
    assert_eq!(HabitCheckIn::key(&habit, day()), "h1:2026-08-07");
}

#[test]
fn recording_the_same_day_twice_produces_the_same_key_so_it_corrects_rather_than_appends() {
    let habit = HabitId::new("h1");
    let first = HabitCheckIn::record(RecordCheckIn {
        habit: habit.clone(), date: day(), outcome: CheckInOutcome::Done, clock: &clock(),
    });
    let corrected = HabitCheckIn::record(RecordCheckIn {
        habit: habit.clone(), date: day(), outcome: CheckInOutcome::Skipped, clock: &clock(),
    });
    assert_eq!(first.id, corrected.id);
    assert_eq!(corrected.outcome, CheckInOutcome::Skipped);
}

#[test]
fn outcomes_serialize_as_camel_case_for_the_frontend() {
    assert_eq!(serde_json::to_string(&CheckInOutcome::NotCompleted).unwrap(), r#""notCompleted""#);
}
```

Implementations follow `DailyPlan`'s shape: a `key` associated function, a constructor taking a
request struct, and `#[serde(rename_all = "camelCase")]` on both the struct and the enum.
`CheckInOutcome` has exactly three variants — `Done`, `Skipped`, `NotCompleted` — and no fourth is
ever added; PRODUCT.md forbids scoring.

- [ ] **Step 5: Run, export, commit**

```bash
cargo test -p planning-core
```

Expected: PASS.

```bash
git add crates/planning-core
git commit -m "feat(core): add DailyPlan, WeeklyFocus, and HabitCheckIn with key-enforced uniqueness"
```

---

### Task 2: Recurrence rules

**Files:**
- Create: `crates/planning-core/src/private/recurrence.rs`,
  `crates/planning-core/src/private/recurring_task.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: ids, `Clock`, `Lifecycle`, `DomainError`.
- Produces:

```rust
pub enum Recurrence {
    Daily,
    Weekdays,                          // Monday-Friday
    Weekly { weekday: Weekday },
    MonthlyDay { day: u8 },            // 1..=31, clamped to the last day of shorter months
}
impl Recurrence {
    pub fn occurs_on(&self, date: NaiveDate) -> bool;
    pub fn monthly(day: u8) -> Result<Self, DomainError>;   // rejects 0 and >31
}

pub struct RecurringTask { pub id: RecurringTaskId, pub title: String,
                           pub recurrence: Recurrence, pub lifecycle: Lifecycle,
                           pub starts_on: NaiveDate, pub materialized_through: Option<NaiveDate>,
                           pub created_at: DateTime<Utc> }
impl RecurringTask {
    pub fn create(CreateRecurringTask { title, recurrence, starts_on, clock }) -> Result<Self, DomainError>;
}

pub struct Occurrence { pub id: OccurrenceId, pub rule: RecurringTaskId,
                        pub date: NaiveDate, pub task: TaskId }
impl Occurrence { pub fn key(rule: &RecurringTaskId, date: NaiveDate) -> String; }
```

`DomainError` gains `#[error("a monthly recurrence day must be between 1 and 31")] InvalidMonthDay`.

**Monthly clamping decision:** `MonthlyDay { day: 31 }` occurs on 28 February (29 in a leap year)
and on 30 April. Skipping those months instead would make a "pay rent on the 31st" rule silently
vanish for five months a year. Clamping is the honest reading of the user's intent.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn daily_occurs_every_day() {
        assert!(Recurrence::Daily.occurs_on(date(2026, 8, 9)));  // a Sunday
    }

    #[test]
    fn weekdays_skips_the_weekend() {
        assert!(Recurrence::Weekdays.occurs_on(date(2026, 8, 7)));   // Friday
        assert!(!Recurrence::Weekdays.occurs_on(date(2026, 8, 8)));  // Saturday
        assert!(!Recurrence::Weekdays.occurs_on(date(2026, 8, 9)));  // Sunday
        assert!(Recurrence::Weekdays.occurs_on(date(2026, 8, 10)));  // Monday
    }

    #[test]
    fn weekly_occurs_on_its_weekday_only() {
        let rule = Recurrence::Weekly { weekday: Weekday::Thu };
        assert!(rule.occurs_on(date(2026, 8, 6)));
        assert!(!rule.occurs_on(date(2026, 8, 7)));
        assert!(rule.occurs_on(date(2026, 8, 13)));
    }

    #[test]
    fn monthly_clamps_to_the_last_day_of_shorter_months() {
        let rule = Recurrence::monthly(31).unwrap();
        assert!(rule.occurs_on(date(2026, 8, 31)));
        assert!(!rule.occurs_on(date(2026, 8, 30)));
        // 2026 is not a leap year: February has 28 days.
        assert!(rule.occurs_on(date(2026, 2, 28)));
        assert!(rule.occurs_on(date(2026, 4, 30)));
        assert!(!rule.occurs_on(date(2026, 4, 29)));
        // A leap year moves the clamp.
        assert!(rule.occurs_on(date(2028, 2, 29)));
        assert!(!rule.occurs_on(date(2028, 2, 28)));
    }

    #[test]
    fn monthly_rejects_impossible_days() {
        assert!(Recurrence::monthly(0).is_err());
        assert!(Recurrence::monthly(32).is_err());
        assert!(Recurrence::monthly(1).is_ok());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p planning-core recurrence
```

Expected: FAIL — `cannot find enum 'Recurrence'`.

- [ ] **Step 3: Implement `recurrence.rs`**

```rust
use super::domain_error::DomainError;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// The recurrence patterns agreed for v1. A rule is a factory: editing it never
/// touches occurrences that already materialized (ADR 0002).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Recurrence {
    Daily,
    Weekdays,
    Weekly { weekday: Weekday },
    MonthlyDay { day: u8 },
}

impl Recurrence {
    pub fn monthly(day: u8) -> Result<Self, DomainError> {
        if day == 0 || day > 31 {
            return Err(DomainError::InvalidMonthDay);
        }
        Ok(Self::MonthlyDay { day })
    }

    pub fn occurs_on(&self, date: NaiveDate) -> bool {
        match self {
            Self::Daily => true,
            Self::Weekdays => !matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            Self::Weekly { weekday } => date.weekday() == *weekday,
            Self::MonthlyDay { day } => date.day() == effective_day(*day, date),
        }
    }
}

/// "The 31st" in a 30-day month means the 30th, not "skipped". Skipping would
/// make a monthly rule silently vanish for five months a year.
fn effective_day(wanted: u8, in_month_of: NaiveDate) -> u32 {
    let last = last_day_of_month(in_month_of);
    u32::from(wanted).min(last)
}

fn last_day_of_month(date: NaiveDate) -> u32 {
    let first_of_this = date.with_day(1).expect("day 1 always exists");
    let first_of_next = match first_of_this.month() {
        12 => NaiveDate::from_ymd_opt(first_of_this.year() + 1, 1, 1),
        month => NaiveDate::from_ymd_opt(first_of_this.year(), month + 1, 1),
    }
    .expect("the first of the next month always exists");
    (first_of_next - Duration::days(1)).day()
}
```

- [ ] **Step 4: Implement `recurring_task.rs`**

```rust
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::{OccurrenceId, RecurringTaskId, TaskId};
use super::lifecycle::Lifecycle;
use super::recurrence::Recurrence;
use super::task::clean_title;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateRecurringTask<'a> {
    pub title: String,
    pub recurrence: Recurrence,
    pub starts_on: NaiveDate,
    pub clock: &'a dyn Clock,
}

/// A factory for Tasks, not a Task. Archiving it stops future materialization and
/// leaves every occurrence already produced untouched (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTask {
    pub id: RecurringTaskId,
    pub title: String,
    pub recurrence: Recurrence,
    pub lifecycle: Lifecycle,
    pub starts_on: NaiveDate,
    /// Fast path only. Correctness comes from `Occurrence`'s key, not from this.
    pub materialized_through: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl RecurringTask {
    pub fn create(request: CreateRecurringTask<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: RecurringTaskId::generate(),
            title: clean_title(request.title)?,
            recurrence: request.recurrence,
            lifecycle: Lifecycle::Active,
            starts_on: request.starts_on,
            materialized_through: None,
            created_at: request.clock.now(),
        })
    }
}

/// Proof that one rule produced one Task on one date. Its key makes a duplicate
/// structurally impossible, so reopening the app cannot double-generate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Occurrence {
    pub id: OccurrenceId,
    pub rule: RecurringTaskId,
    pub date: NaiveDate,
    pub task: TaskId,
}

impl Occurrence {
    pub fn key(rule: &RecurringTaskId, date: NaiveDate) -> String {
        format!("{rule}:{}", date.format("%Y-%m-%d"))
    }
}
```

- [ ] **Step 5: Run, export, commit**

```bash
cargo test -p planning-core && cargo clippy -p planning-core --all-targets -- -D warnings
```

Expected: PASS.

```bash
git add crates/planning-core
git commit -m "feat(core): add recurrence patterns and occurrence records"
```

---

### Task 3: Idempotent materialization

**Files:**
- Create: `crates/planning-app/src/private/materialization.rs`
- Modify: `crates/planning-app/src/private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: `RecurringTask`, `Occurrence`, `Task::create`, the store helpers from plan 0004.
- Produces:
  - `create_recurring_task(&self, NewRecurringTask { title, recurrence }) -> Result<RecurringTask, AppError>`
    — `starts_on` defaults to today in the home zone.
  - `archive_recurring_task(&self, &RecurringTaskId)` / `restore_recurring_task`
  - `materialize_due(&self) -> Result<Vec<Task>, AppError>` — generates every missing occurrence
    from `max(starts_on, today - CATCH_UP_DAYS)` through today, for active rules only.
  - `PlanningApp::CATCH_UP_DAYS: i64 = 31`
  - `recurring_tasks(&self) -> Result<Vec<RecurringTask>, AppError>`

**This task claims acceptance criterion A5.**

The catch-up window is capped at 31 days on purpose: reopening the app after a six-month absence
must not dump 180 stale Tasks into the Task Pool. The cap is a product decision, not an
optimization, and it must be stated in the architecture doc.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app_at;
    use chrono::{Duration, TimeZone};
    use planning_core::Recurrence;

    /// `ready_app_at` is `ready_app` from plan 0004 with a caller-supplied instant;
    /// add it to test_support.rs and express `ready_app` in terms of it.
    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn reopening_the_app_never_duplicates_an_occurrence() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();

        let first = app.materialize_due().await.unwrap();
        assert_eq!(first.len(), 1);

        // Three more "app opens" on the same day.
        for _ in 0..3 {
            assert!(app.materialize_due().await.unwrap().is_empty());
        }
        assert_eq!(app.tasks().await.unwrap().len(), 1, "A5: no duplicates");
    }

    #[tokio::test]
    async fn a_gap_in_usage_catches_up_day_by_day() {
        let (_home, _drive, app, clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();
        app.materialize_due().await.unwrap();

        clock.advance(Duration::days(3));
        let caught_up = app.materialize_due().await.unwrap();
        assert_eq!(caught_up.len(), 3, "the 8th, 9th, and 10th");
        assert_eq!(app.tasks().await.unwrap().len(), 4);
    }

    #[tokio::test]
    async fn catch_up_is_capped_so_a_long_absence_does_not_flood_the_task_pool() {
        let (_home, _drive, app, clock) = app_on(7).await;
        app.create_recurring_task(NewRecurringTask {
            title: "Morning pages".into(),
            recurrence: Recurrence::Daily,
        })
        .await
        .unwrap();

        clock.advance(Duration::days(200));
        let caught_up = app.materialize_due().await.unwrap();
        assert_eq!(caught_up.len() as i64, PlanningApp::CATCH_UP_DAYS + 1);
    }

    #[tokio::test]
    async fn archiving_a_rule_stops_future_occurrences_and_keeps_past_ones() {
        let (_home, _drive, app, clock) = app_on(7).await;
        let rule = app
            .create_recurring_task(NewRecurringTask {
                title: "Morning pages".into(),
                recurrence: Recurrence::Daily,
            })
            .await
            .unwrap();
        app.materialize_due().await.unwrap();

        app.archive_recurring_task(&rule.id).await.unwrap();
        clock.advance(Duration::days(2));

        assert!(app.materialize_due().await.unwrap().is_empty());
        assert_eq!(app.tasks().await.unwrap().len(), 1, "the occurrence already made survives");
    }

    #[tokio::test]
    async fn a_materialized_occurrence_is_an_ordinary_task_unaffected_by_later_rule_edits() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let rule = app
            .create_recurring_task(NewRecurringTask {
                title: "Morning pages".into(),
                recurrence: Recurrence::Daily,
            })
            .await
            .unwrap();
        let produced = app.materialize_due().await.unwrap();
        let task = &produced[0];

        app.rename_recurring_task(&rule.id, "Evening pages".into()).await.unwrap();

        assert_eq!(
            app.task(&task.id).await.unwrap().unwrap().title,
            "Morning pages",
            "the occurrence is its own Task now"
        );
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p planning-app materialization
```

Expected: FAIL — `no method named 'create_recurring_task'`.

- [ ] **Step 3: Implement `materialization.rs`**

```rust
use super::error::AppError;
use super::service::PlanningApp;
use chrono::{Duration, NaiveDate};
use planning_core::{
    CreateRecurringTask, CreateTask, Lifecycle, Occurrence, OccurrenceId, Recurrence,
    RecurringTask, RecurringTaskId, Task, TaskId,
};

pub struct NewRecurringTask {
    pub title: String,
    pub recurrence: Recurrence,
}

impl PlanningApp {
    /// How far back materialization will catch up. Capped so that reopening the
    /// app after months away does not dump hundreds of stale Tasks into the Task
    /// Pool — a product decision, not an optimization.
    pub const CATCH_UP_DAYS: i64 = 31;

    pub async fn create_recurring_task(
        &self,
        request: NewRecurringTask,
    ) -> Result<RecurringTask, AppError> {
        let starts_on = self.calendar()?.today(self.clock.as_ref());
        let rule = RecurringTask::create(CreateRecurringTask {
            title: request.title,
            recurrence: request.recurrence,
            starts_on,
            clock: self.clock.as_ref(),
        })?;
        self.store(RecurringTaskId::TABLE, rule.id.as_str(), &rule).await?;
        Ok(rule)
    }

    /// Generates every missing occurrence up to today. Safe to call on every app
    /// open: the `Occurrence` key makes a second attempt a no-op (A5).
    pub async fn materialize_due(&self) -> Result<Vec<Task>, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let mut produced = Vec::new();
        for rule in self.active_rules().await? {
            let made = self.materialize_rule(&rule, today).await?;
            produced.extend(made);
        }
        Ok(produced)
    }

    async fn active_rules(&self) -> Result<Vec<RecurringTask>, AppError> {
        Ok(self
            .recurring_tasks()
            .await?
            .into_iter()
            .filter(|rule| rule.lifecycle.is_active())
            .collect())
    }

    async fn materialize_rule(
        &self,
        rule: &RecurringTask,
        today: NaiveDate,
    ) -> Result<Vec<Task>, AppError> {
        let mut produced = Vec::new();
        let mut date = first_candidate(rule, today);
        while date <= today {
            if let Some(task) = self.materialize_one(rule, date).await? {
                produced.push(task);
            }
            date += Duration::days(1);
        }
        self.record_progress(rule, today).await?;
        Ok(produced)
    }

    /// Returns None when this rule does not occur on `date` or already has.
    async fn materialize_one(
        &self,
        rule: &RecurringTask,
        date: NaiveDate,
    ) -> Result<Option<Task>, AppError> {
        if !rule.recurrence.occurs_on(date) {
            return Ok(None);
        }
        let key = Occurrence::key(&rule.id, date);
        let existing: Option<Occurrence> = self.load_one(OccurrenceId::TABLE, &key).await?;
        if existing.is_some() {
            return Ok(None);
        }

        let task = Task::create(CreateTask {
            title: rule.title.clone(),
            clock: self.clock.as_ref(),
        })?;
        self.store(TaskId::TABLE, task.id.as_str(), &task).await?;

        let occurrence = Occurrence {
            id: OccurrenceId::new(key.clone()),
            rule: rule.id.clone(),
            date,
            task: task.id.clone(),
        };
        self.store(OccurrenceId::TABLE, &key, &occurrence).await?;
        Ok(Some(task))
    }

    async fn record_progress(
        &self,
        rule: &RecurringTask,
        today: NaiveDate,
    ) -> Result<(), AppError> {
        self.mutate::<RecurringTask>((RecurringTaskId::TABLE, rule.id.to_string()), |found| {
            found.materialized_through = Some(today);
        })
        .await?;
        Ok(())
    }

    pub async fn recurring_tasks(&self) -> Result<Vec<RecurringTask>, AppError> {
        self.load_all(RecurringTaskId::TABLE).await
    }
}

/// Resume from where we left off, never earlier than the rule's start and never
/// more than CATCH_UP_DAYS back.
fn first_candidate(rule: &RecurringTask, today: NaiveDate) -> NaiveDate {
    let resume = rule
        .materialized_through
        .map(|through| through + Duration::days(1))
        .unwrap_or(rule.starts_on);
    let floor = today - Duration::days(PlanningApp::CATCH_UP_DAYS);
    resume.max(rule.starts_on).max(floor)
}
```

Add `archive_recurring_task`, `restore_recurring_task`, and `rename_recurring_task` following the
`set_task_lifecycle` shape from plan 0004. Split them into a sibling
`recurring_task_lifecycle.rs` if `materialization.rs` passes 200 lines.

- [ ] **Step 4: Run, commit**

```bash
cargo test -p planning-app materialization
```

Expected: PASS — 5 tests. **A5 is now proven.**

```bash
git add crates/planning-app
git commit -m "feat(app): add duplicate-safe recurring task materialization"
```

---

### Task 4: Weekly Focus use cases

**Files:**
- Create: `crates/planning-app/src/private/weekly_focus_use_cases.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:
  - `weekly_focus(&self, CalendarWeek) -> Result<WeeklyFocus, AppError>` — creates an empty focus
    on first read rather than returning `None`.
  - `current_weekly_focus(&self) -> Result<WeeklyFocus, AppError>`
  - `add_to_focus(&self, FocusChange { week, task })` — refuses an archived Task with
    `AppError::NotSelectable`
  - `remove_from_focus(&self, FocusChange { week, task })`
  - `AppError::NotSelectable { reason: &'static str }`

A Weekly Focus can be adjusted **at any time**, not only during a Weekly Review — the Library and
the Weekly Review call the identical methods. That is half of acceptance criterion A8.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn a_focus_is_created_empty_on_first_read_and_survives_reload() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    assert!(app.weekly_focus(week).await.unwrap().tasks.is_empty());

    let task = app.create_task("Prepare portfolio".into()).await.unwrap();
    app.add_to_focus(FocusChange { week, task: task.id.clone() }).await.unwrap();
    assert_eq!(app.weekly_focus(week).await.unwrap().tasks, vec![task.id]);
}

#[tokio::test]
async fn selecting_into_a_focus_never_removes_the_task_from_the_pool() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let task = app.create_task("Prepare portfolio".into()).await.unwrap();
    app.add_to_focus(FocusChange { week, task: task.id.clone() }).await.unwrap();

    assert_eq!(app.tasks().await.unwrap().len(), 1);
    assert!(app.task(&task.id).await.unwrap().unwrap().lifecycle.is_active());
}

#[tokio::test]
async fn an_archived_task_cannot_be_newly_selected_into_a_focus() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let task = app.create_task("Old idea".into()).await.unwrap();
    app.archive_task(&task.id).await.unwrap();

    assert!(matches!(
        app.add_to_focus(FocusChange { week, task: task.id }).await.unwrap_err(),
        AppError::NotSelectable { .. }
    ));
}

#[tokio::test]
async fn archiving_a_task_already_in_a_focus_leaves_the_entry_in_place() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let task = app.create_task("Prepare portfolio".into()).await.unwrap();
    app.add_to_focus(FocusChange { week, task: task.id.clone() }).await.unwrap();

    app.archive_task(&task.id).await.unwrap();

    // Forward-only: the focus is not rewritten (ADR 0002).
    assert_eq!(app.weekly_focus(week).await.unwrap().tasks, vec![task.id]);
}
```

`app.clock_ref()` is a small accessor returning `&dyn Clock`; add it to `service.rs`.

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-app weekly_focus
```

```rust
use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{CalendarWeek, StartFocus, TaskId, WeeklyFocus, WeeklyFocusId};

pub struct FocusChange {
    pub week: CalendarWeek,
    pub task: TaskId,
}

impl PlanningApp {
    /// Creates an empty focus on first read so callers never branch on "does one exist".
    pub async fn weekly_focus(&self, week: CalendarWeek) -> Result<WeeklyFocus, AppError> {
        let key = WeeklyFocus::key(week);
        if let Some(found) = self.load_one::<WeeklyFocus>(WeeklyFocusId::TABLE, &key).await? {
            return Ok(found);
        }
        let created = WeeklyFocus::start(StartFocus { week, clock: self.clock.as_ref() });
        self.store(WeeklyFocusId::TABLE, &key, &created).await?;
        Ok(created)
    }

    pub async fn current_weekly_focus(&self) -> Result<WeeklyFocus, AppError> {
        let week = self.calendar()?.current_week(self.clock.as_ref());
        self.weekly_focus(week).await
    }

    /// Archived Tasks cannot be newly selected, but ones already present stay —
    /// this method is the "newly" half of that rule (ADR 0002).
    pub async fn add_to_focus(&self, change: FocusChange) -> Result<(), AppError> {
        self.require_selectable_task(&change.task).await?;
        let key = WeeklyFocus::key(change.week);
        self.weekly_focus(change.week).await?;
        self.mutate::<WeeklyFocus>((WeeklyFocusId::TABLE, key), |focus| {
            focus.add(change.task.clone());
        })
        .await?;
        Ok(())
    }

    pub async fn remove_from_focus(&self, change: FocusChange) -> Result<(), AppError> {
        let key = WeeklyFocus::key(change.week);
        self.mutate::<WeeklyFocus>((WeeklyFocusId::TABLE, key), |focus| {
            focus.remove(&change.task);
        })
        .await?;
        Ok(())
    }

    pub(crate) async fn require_selectable_task(&self, task: &TaskId) -> Result<(), AppError> {
        let found = self
            .task(task)
            .await?
            .ok_or(AppError::NotFound { table: "task", id: task.to_string() })?;
        if !found.lifecycle.is_active() {
            return Err(AppError::NotSelectable { reason: "the task is archived" });
        }
        Ok(())
    }
}
```

Add `NotSelectable { reason: &'static str }` to `AppError`.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app weekly_focus
```

Expected: PASS — 4 tests.

```bash
git add crates/planning-app
git commit -m "feat(app): add Weekly Focus editable at any time"
```

---

### Task 5: Opening today's Daily Plan

**Files:**
- Create: `crates/planning-app/src/private/daily_plan_use_cases.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:
  - `open_plan(&self, NaiveDate) -> Result<DailyPlan, AppError>` — creates on first open, seeding
    habits from the pinned-and-due set; returns the existing plan afterwards, **unchanged**.
  - `open_today(&self) -> Result<DailyPlan, AppError>` — materializes recurring tasks first, then
    `open_plan(today)`.
  - `has_plan_for(&self, NaiveDate) -> Result<bool, AppError>` — **read-only**; plan 0008's
    launcher calls exactly this.
  - `select_into_plan(&self, PlanChange { date, task })` — refuses archived
  - `remove_from_plan(&self, PlanChange { date, task })`
  - `reorder_plan(&self, ReorderPlan { date, order })` — `AppError::InvalidOrder` if not a permutation
  - `add_habit_to_plan(&self, PlanHabitChange { date, habit })` — the manual one-day addition
  - `quick_add_task(&self, title: String) -> Result<Task, AppError>` — creates **and** selects into
    today's plan

**This task claims acceptance criterion A2 and the second half of A6.**

- [ ] **Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn opening_twice_returns_the_same_plan_without_reseeding() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();

    let first = app.open_today().await.unwrap();
    assert_eq!(first.habits, vec![habit.id.clone()]);

    app.set_habit_pinned(SetPinned { habit: &habit.id, pinned: false }).await.unwrap();
    let second = app.open_today().await.unwrap();
    assert_eq!(second.habits, vec![habit.id], "forward-only: today's plan is not rewritten");
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

    assert_eq!(app.open_today().await.unwrap().habits, vec![weekday_only.id.clone()]);

    clock.advance(Duration::days(1));
    assert!(app.open_today().await.unwrap().habits.is_empty(), "Saturday is not a cadence day");
}

#[tokio::test]
async fn an_unpinned_habit_is_never_seeded_into_a_new_plan() {
    let (_home, _drive, app, clock) = app_on(7).await;
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    app.set_habit_pinned(SetPinned { habit: &habit.id, pinned: false }).await.unwrap();

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
        app.select_into_plan(PlanChange { date: today, task: id.clone() }).await.unwrap();
    }
    // Selecting twice is a no-op, not a duplicate (A2).
    app.select_into_plan(PlanChange { date: today, task: ids[0].clone() }).await.unwrap();
    assert_eq!(app.open_today().await.unwrap().tasks.len(), 3);

    app.reorder_plan(ReorderPlan {
        date: today,
        order: vec![ids[2].clone(), ids[0].clone(), ids[1].clone()],
    })
    .await
    .unwrap();
    assert_eq!(app.open_today().await.unwrap().tasks[0], ids[2]);

    app.remove_from_plan(PlanChange { date: today, task: ids[1].clone() }).await.unwrap();
    assert_eq!(app.open_today().await.unwrap().tasks.len(), 2);
    assert_eq!(app.tasks().await.unwrap().len(), 3, "removal never touches the Task Pool");

    app.complete_task(&ids[0]).await.unwrap();
    assert!(app.task(&ids[0]).await.unwrap().unwrap().completion.is_complete());
}

#[tokio::test]
async fn a_bad_reorder_is_rejected_without_changing_the_plan() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let task = app.create_task("One".into()).await.unwrap();
    app.select_into_plan(PlanChange { date: today, task: task.id.clone() }).await.unwrap();

    let error = app
        .reorder_plan(ReorderPlan { date: today, order: vec![TaskId::new("ghost")] })
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
    app.select_into_plan(PlanChange { date: today, task: task.id.clone() }).await.unwrap();

    app.archive_task(&task.id).await.unwrap();

    let plan = app.open_today().await.unwrap();
    assert_eq!(plan.tasks, vec![task.id.clone()], "the entry stays");

    // Still completable while archived — completion is never gated on the plan.
    app.complete_task(&task.id).await.unwrap();
    assert!(app.task(&task.id).await.unwrap().unwrap().completion.is_complete());

    // But it cannot be newly selected into another day.
    let tomorrow = today + Duration::days(1);
    assert!(matches!(
        app.select_into_plan(PlanChange { date: tomorrow, task: task.id }).await.unwrap_err(),
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
    assert!(!app.has_plan_for(today).await.unwrap(), "asking must not create a plan");

    app.open_today().await.unwrap();
    assert!(app.has_plan_for(today).await.unwrap());
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p planning-app daily_plan
```

Expected: FAIL — `no method named 'open_today'`.

- [ ] **Step 3: Implement `daily_plan_use_cases.rs`**

```rust
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
        let created = DailyPlan::start(StartPlan { date, habits, clock: self.clock.as_ref() });
        self.store(DailyPlanId::TABLE, &key, &created).await?;
        Ok(created)
    }

    /// Read-only existence check. Plan 0008's launcher calls exactly this and must
    /// never cause a plan to be created as a side effect.
    pub async fn has_plan_for(&self, date: NaiveDate) -> Result<bool, AppError> {
        let found: Option<DailyPlan> =
            self.load_one(DailyPlanId::TABLE, &DailyPlan::key(date)).await?;
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
        let mut plan: DailyPlan = self
            .load_one(DailyPlanId::TABLE, &key)
            .await?
            .ok_or(AppError::NotFound { table: "daily_plan", id: key.clone() })?;
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
        self.select_into_plan(PlanChange { date: today, task: task.id.clone() }).await?;
        Ok(task)
    }
}
```

Add `#[error("the proposed order is not a permutation of the plan")] InvalidOrder` to `AppError`.
This file will be close to 200 lines — if it exceeds, move `select_into_plan`, `remove_from_plan`,
`reorder_plan`, and `add_habit_to_plan` into `daily_plan_editing.rs`.

- [ ] **Step 4: Run, commit**

```bash
cargo test -p planning-app daily_plan
```

Expected: PASS — 8 tests. **A2 and A6 are now fully proven.**

```bash
git add crates/planning-app
git commit -m "feat(app): add Daily Plan opening, selection, ordering, and quick add"
```

---

### Task 6: Habit check-ins

**Files:**
- Create: `crates/planning-app/src/private/check_in_use_cases.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:
  - `record_check_in(&self, CheckInRequest { habit, date, outcome }) -> Result<HabitCheckIn, AppError>`
    — works for **any** date, past or present, and overwrites rather than appends.
  - `check_in_for(&self, habit: &HabitId, date: NaiveDate) -> Result<Option<HabitCheckIn>, AppError>`
  - `check_ins_between(&self, DateRange { from, to }) -> Result<Vec<HabitCheckIn>, AppError>` —
    plan 0006's Weekly Report summary calls this.

**This task claims the check-in half of acceptance criterion A3.**

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn all_three_outcomes_are_recordable_and_nothing_else_exists() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();

    for outcome in [CheckInOutcome::Done, CheckInOutcome::Skipped, CheckInOutcome::NotCompleted] {
        app.record_check_in(CheckInRequest { habit: habit.id.clone(), date: today, outcome })
            .await
            .unwrap();
        assert_eq!(app.check_in_for(&habit.id, today).await.unwrap().unwrap().outcome, outcome);
    }
}

#[tokio::test]
async fn correcting_a_past_day_replaces_the_outcome_rather_than_adding_one() {
    let (_home, _drive, app, clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    let yesterday = today - Duration::days(1);

    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: yesterday, outcome: CheckInOutcome::NotCompleted,
    })
    .await
    .unwrap();

    clock.advance(Duration::days(5));
    // Still correctable days later (ADR 0002).
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: yesterday, outcome: CheckInOutcome::Done,
    })
    .await
    .unwrap();

    assert_eq!(
        app.check_in_for(&habit.id, yesterday).await.unwrap().unwrap().outcome,
        CheckInOutcome::Done
    );
    assert_eq!(
        app.check_ins_between(DateRange { from: yesterday, to: yesterday }).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn an_archived_habit_can_still_be_checked_in_for_a_day_it_already_appears_in() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    app.open_today().await.unwrap();

    app.archive_habit(&habit.id).await.unwrap();

    // A6: the entry remains and stays completable.
    assert!(app.open_today().await.unwrap().habits.contains(&habit.id));
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: today, outcome: CheckInOutcome::Done,
    })
    .await
    .unwrap();
    assert!(app.check_in_for(&habit.id, today).await.unwrap().is_some());
}
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-app check_in
```

```rust
use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{
    CheckInOutcome, HabitCheckIn, HabitCheckInId, HabitId, RecordCheckIn,
};

pub struct CheckInRequest {
    pub habit: HabitId,
    pub date: NaiveDate,
    pub outcome: CheckInOutcome,
}

pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl PlanningApp {
    /// Records or corrects one Habit's outcome for one day. Deliberately does NOT
    /// check the habit's lifecycle or cadence: archived habits already in a plan
    /// stay completable, and any past day stays correctable (ADR 0002).
    pub async fn record_check_in(
        &self,
        request: CheckInRequest,
    ) -> Result<HabitCheckIn, AppError> {
        let record = HabitCheckIn::record(RecordCheckIn {
            habit: request.habit,
            date: request.date,
            outcome: request.outcome,
            clock: self.clock.as_ref(),
        });
        self.store(HabitCheckInId::TABLE, record.id.as_str(), &record).await?;
        Ok(record)
    }

    pub async fn check_in_for(
        &self,
        habit: &HabitId,
        date: NaiveDate,
    ) -> Result<Option<HabitCheckIn>, AppError> {
        self.load_one(HabitCheckInId::TABLE, &HabitCheckIn::key(habit, date)).await
    }

    pub async fn check_ins_between(
        &self,
        range: DateRange,
    ) -> Result<Vec<HabitCheckIn>, AppError> {
        Ok(self
            .load_all::<HabitCheckIn>(HabitCheckInId::TABLE)
            .await?
            .into_iter()
            .filter(|found| found.date >= range.from && found.date <= range.to)
            .collect())
    }
}
```

`check_in_for` takes three parameters counting `&self`, which is within the limit.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app check_in
```

Expected: PASS — 3 tests.

```bash
git add crates/planning-app
git commit -m "feat(app): add correctable habit check-ins"
```

---

### Task 7: Plan read models

**Files:**
- Create: `crates/planning-app/src/private/plan_views.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:

```rust
pub struct PlanTaskView { id, title, state: TaskState, importance, urgency,
                          deadline, overdue, archived, position: u32 }
pub struct PlanHabitView { id, title, cadence, archived, unpinned, outcome: Option<CheckInOutcome> }
pub struct DailyPlanView { date: NaiveDate, week: CalendarWeek,
                           tasks: Vec<PlanTaskView>, habits: Vec<PlanHabitView> }
pub struct TaskPoolView { focus: Vec<TaskView>, rest: Vec<TaskView> }

PlanningApp::today_view(&self) -> Result<DailyPlanView, AppError>
PlanningApp::plan_view(&self, NaiveDate) -> Result<DailyPlanView, AppError>
PlanningApp::task_pool(&self) -> Result<TaskPoolView, AppError>
```

`archived` and `unpinned` are the honest-state markers PRODUCT.md requires: an entry whose entity
was archived or unpinned after the plan was made still renders, flagged. `TaskPoolView` splits
Weekly-Focus tasks from the rest so the UI can show focus first without inventing an ordering rule.

**This task completes acceptance criterion A3** — pinned habits appear only on cadence days, with
their outcome projected.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn a_plan_entry_whose_entity_was_archived_renders_flagged_and_ordered() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let kept = app.create_task("Keep".into()).await.unwrap();
    let archived = app.create_task("Archive me".into()).await.unwrap();
    for id in [&kept.id, &archived.id] {
        app.select_into_plan(PlanChange { date: today, task: id.clone() }).await.unwrap();
    }
    app.archive_task(&archived.id).await.unwrap();

    let view = app.today_view().await.unwrap();
    assert_eq!(view.tasks.len(), 2, "the archived entry is shown, not hidden");
    assert_eq!(view.tasks[0].position, 0);
    assert_eq!(view.tasks[1].position, 1);
    let flagged = view.tasks.iter().find(|task| task.id == archived.id).unwrap();
    assert!(flagged.archived);
    assert_eq!(flagged.state, TaskState::Archived);
}

#[tokio::test]
async fn an_unpinned_habit_still_in_todays_plan_is_flagged_and_shows_its_outcome() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let today = app.calendar().unwrap().today(app.clock_ref());
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    app.open_today().await.unwrap();
    app.set_habit_pinned(SetPinned { habit: &habit.id, pinned: false }).await.unwrap();
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: today, outcome: CheckInOutcome::Skipped,
    })
    .await
    .unwrap();

    let view = app.today_view().await.unwrap();
    assert_eq!(view.habits.len(), 1);
    assert!(view.habits[0].unpinned);
    assert_eq!(view.habits[0].outcome, Some(CheckInOutcome::Skipped));
    assert_eq!(view.week.label(), "2026-W32");
}

#[tokio::test]
async fn the_task_pool_puts_weekly_focus_tasks_first_and_excludes_closed_ones() {
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let focused = app.create_task("Focused".into()).await.unwrap();
    let other = app.create_task("Other".into()).await.unwrap();
    let done = app.create_task("Done".into()).await.unwrap();
    let gone = app.create_task("Archived".into()).await.unwrap();
    app.add_to_focus(FocusChange { week, task: focused.id.clone() }).await.unwrap();
    app.complete_task(&done.id).await.unwrap();
    app.archive_task(&gone.id).await.unwrap();

    let pool = app.task_pool().await.unwrap();
    assert_eq!(pool.focus.iter().map(|t| t.id.clone()).collect::<Vec<_>>(), vec![focused.id]);
    assert_eq!(pool.rest.iter().map(|t| t.id.clone()).collect::<Vec<_>>(), vec![other.id]);
}
```

- [ ] **Step 2: Run to verify it fails, then implement `plan_views.rs`**

```bash
cargo test -p planning-app plan_views
```

```rust
use super::error::AppError;
use super::service::PlanningApp;
use super::views::{TaskState, TaskView};
use chrono::NaiveDate;
use planning_core::{
    Cadence, CalendarWeek, CheckInOutcome, Classification, DailyPlan, Habit, HabitId, Task, TaskId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTaskView {
    pub id: TaskId,
    pub title: String,
    pub state: TaskState,
    pub importance: Classification,
    pub urgency: Classification,
    pub deadline: Option<NaiveDate>,
    pub overdue: bool,
    pub archived: bool,
    pub position: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanHabitView {
    pub id: HabitId,
    pub title: String,
    pub cadence: Cadence,
    pub archived: bool,
    /// True when the Habit was unpinned after this plan was made. The entry stays
    /// and stays completable — the UI shows the truth (PRODUCT.md).
    pub unpinned: bool,
    pub outcome: Option<CheckInOutcome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPlanView {
    pub date: NaiveDate,
    pub week: CalendarWeek,
    pub tasks: Vec<PlanTaskView>,
    pub habits: Vec<PlanHabitView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPoolView {
    pub focus: Vec<TaskView>,
    pub rest: Vec<TaskView>,
}

impl PlanningApp {
    pub async fn today_view(&self) -> Result<DailyPlanView, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        self.open_plan(today).await?;
        self.plan_view(today).await
    }

    /// Resolves every stored id against current entity state. An entity archived
    /// after the plan was made shows up flagged rather than disappearing.
    pub async fn plan_view(&self, date: NaiveDate) -> Result<DailyPlanView, AppError> {
        let plan = self.open_plan(date).await?;
        Ok(DailyPlanView {
            date,
            week: CalendarWeek::containing(date),
            tasks: self.project_plan_tasks(&plan).await?,
            habits: self.project_plan_habits(&plan).await?,
        })
    }
}
```

Write `project_plan_tasks` and `project_plan_habits` as private methods in a sibling
`plan_projection.rs`, each under 30 lines. `project_plan_tasks` loads every Task once into a
`HashMap<TaskId, Task>` and maps the plan's ordered ids through it, assigning `position` from the
index — never an N+1 load per entry. `project_plan_habits` does the same for Habits plus a single
`check_ins_between { from: date, to: date }` call.

An id in a plan with no matching entity (only possible if the database was edited externally) is
skipped rather than erroring — add a test for it.

`task_pool` loads the current Weekly Focus, then partitions open, active Tasks into `focus` and
`rest`, preserving the focus's own order for the first list.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS. **A3 is now proven.**

```bash
git add crates/planning-app
git commit -m "feat(app): add Daily Plan and Task Pool read models with honest state flags"
```

---

### Task 8: Tauri commands and the TypeScript mirror

**Files:**
- Create: `src-tauri/src/private/plan_commands.rs`
- Modify: `src-tauri/src/lib.rs`, `src/lib/domain/index.ts`, `src/lib/api/index.ts`
- Test: `src/lib/api/plan.test.ts`, JSON-shape tests in `plan_views.rs`

**Interfaces:**

| Command | Args | Returns |
|---------|------|---------|
| `today_view` | — | `DailyPlanView` |
| `task_pool` | — | `TaskPoolView` |
| `select_into_plan` / `remove_from_plan` | `date`, `task` | `void` |
| `reorder_plan` | `date`, `order: string[]` | `void` |
| `add_habit_to_plan` | `date`, `habit` | `void` |
| `quick_add_task` | `title` | `Task` |
| `record_check_in` | `habit`, `date`, `outcome` | `void` |
| `weekly_focus` | `week: string` | `WeeklyFocus` |
| `add_to_focus` / `remove_from_focus` | `week`, `task` | `void` |
| `create_recurring_task` | `title`, `recurrence` | `RecurringTask` |
| `recurring_tasks` | — | `RecurringTask[]` |
| `archive_recurring_task` / `restore_recurring_task` | `rule` | `void` |

- [ ] **Step 1: Add JSON-shape tests for `DailyPlanView` and `PlanHabitView`**

Follow the `TaskView` example from plan 0004 Task 10. Assert the exact serialized string, including
`"week":"2026-W32"` (`CalendarWeek` serializes as its label) and `"outcome":"notCompleted"`.

- [ ] **Step 2: Write the commands, register them, extend the TypeScript mirror**

Add to `src/lib/domain/index.ts`:

```ts
export type CheckInOutcome = 'done' | 'skipped' | 'notCompleted';

export type Recurrence =
  | { kind: 'daily' }
  | { kind: 'weekdays' }
  | { kind: 'weekly'; weekday: Weekday }
  | { kind: 'monthlyDay'; day: number };

export interface PlanTaskView {
  id: string;
  title: string;
  state: TaskState;
  importance: Classification;
  urgency: Classification;
  deadline: string | null;
  overdue: boolean;
  archived: boolean;
  position: number;
}

export interface PlanHabitView {
  id: string;
  title: string;
  cadence: Cadence;
  archived: boolean;
  unpinned: boolean;
  outcome: CheckInOutcome | null;
}

export interface DailyPlanView {
  date: string;
  week: string;
  tasks: PlanTaskView[];
  habits: PlanHabitView[];
}

export interface TaskPoolView {
  focus: TaskView[];
  rest: TaskView[];
}
```

- [ ] **Step 3: Run the full gate and commit**

```bash
npm run check && fallow audit
```

```bash
git add src-tauri src/lib
git commit -m "feat: expose the Daily Plan API to the frontend"
```

---

### Task 9: Documentation

**Files:**
- Create: `docs/architecture/daily-planning.md`, `docs/flows/opening-todays-plan.md`,
  `docs/flows/archiving-a-habit-already-in-a-plan.md`,
  `docs/lessons-learned/record-keys-as-invariants.md`
- Modify: the three README index tables, `docs/live/current-status.md`

- [ ] **Step 1: Write `docs/architecture/daily-planning.md`** (target 90 lines)

Cover: the four key-as-invariant records and what each guarantees; that a Daily Plan stores ids
only and marking is projected; the pinned-and-due seeding rule and that seeding happens **once**,
at creation; that `open_plan` never re-seeds; the `CATCH_UP_DAYS = 31` cap and why it is a product
decision; the monthly-clamping decision; and the fact that `has_plan_for` is read-only because the
launcher depends on it.

- [ ] **Step 2: Write both flow docs**

`opening-todays-plan.md`: Trigger (app opens or the date rolls over) → `open_today` →
materialize → load-or-create → seed pinned-and-due habits → project → Reads/Writes/Side effects →
Common failure modes (store not `Ready`, home zone unset, clock crossing midnight mid-session).

`archiving-a-habit-already-in-a-plan.md`: the flow that answers "why is this still here?" — trigger
(user archives a Habit from the Library), what changes (one `lifecycle` field), what explicitly does
**not** change (today's plan, its check-ins, its associations), and how the entry renders afterwards
(`archived: true`, still checkable). This is the single most surprising behavior in the app, and it
is deliberate.

- [ ] **Step 3: Write `docs/lessons-learned/record-keys-as-invariants.md`**

Topic: choosing the record key to *be* the uniqueness rule — date for a plan, week label for a
focus, `habit:date` for a check-in, `rule:date` for an occurrence. The payoff: "never duplicate on
reopen" (A5) needs no transaction, no query, and no `last_run` bookkeeping to get right; correcting
a past check-in is an upsert rather than a find-then-update. The counter-intuitive part: the
`materialized_through` field is a *performance* hint only, and treating it as the correctness
mechanism is exactly the bug this design avoids.

- [ ] **Step 4: Register everything, update `current-status.md`, commit**

```bash
git add docs
git commit -m "docs: document daily planning, plan flows, and the record-key lesson"
```

---

## Task 10: Verify the plan's own acceptance

- [ ] `npm run check` and `fallow audit` both pass.
- [ ] **A2:** select, reorder, remove, and complete Tasks in a plan without duplication.
- [ ] **A3:** pinned Habits appear only on cadence days and record exactly one of three outcomes.
- [ ] **A5:** calling `materialize_due` four times in a row produces occurrences once.
- [ ] **A6:** archiving a Task or Habit already in today's plan leaves the entry in place, flagged
      and still completable, while blocking selection into tomorrow's plan.
- [ ] `has_plan_for` returns `false` twice in a row without creating a plan.
- [ ] Advancing the clock past midnight and calling `open_today` creates a *new* plan and leaves
      yesterday's untouched.

**Next:** [0006-weekly-review-and-reports.md](0006-weekly-review-and-reports.md) and
[0007-ui-surfaces.md](0007-ui-surfaces.md) may proceed in parallel.
