# Planning Domain & Library API — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](0001-self-planning-app.md) first. Requires
> [0003-storage-and-settings.md](done/0003-storage-and-settings.md) to be complete.

**Goal:** Model Values, Goals, Habits, Tasks, and Associations with an archive-only reversible
lifecycle, forward-only propagation, correctable outcomes, and overdue projection — exposed through
the `planning-app` API and usable from the frontend.

**Architecture:** Entities are plain data in `planning-core` with constructors that enforce the
creation rules; they perform no IO and hold no references. `planning-store` gains one generic
`Records` gateway rather than six near-identical repositories. `planning-app` gains use-case files
and the read-model view types that cross to the UI, where archived/overdue flags are *projected*
rather than stored.

**Key design decision — orthogonal lifecycle:** `CONTEXT.md` describes a Task as "open, completed,
or archived", but those are two independent axes, not three states. A Task carries a `Completion`
*and* a `Lifecycle`. Collapsing them into one enum would make restoring an archived completed Task
ambiguous — it would have to guess whether to come back open or completed. Goals get the same
treatment (`Achievement` × `Lifecycle`). The single-enum `TaskState` still exists, but only as a
*derived view* for the UI.

**Tech Stack:** As plan 0003. No new dependencies.

---

## Global constraints

See [0001-self-planning-app.md](0001-self-planning-app.md#global-constraints). The ones this plan
touches most:

- **Nothing is ever hard-deleted.** Archive is the only removal and it is reversible.
- **Associations never cascade.** Archiving one side leaves the other untouched and keeps the link.
- **Outcomes stay correctable.** Completing a Task is never gated on it being in a Daily Plan.
- **Forward-only.** Nothing in this plan rewrites an existing Daily Plan or Weekly Focus — plan
  0005 relies on that, and the archived-entry marking it needs is a *projection*, added here.

---

## File structure

| File | Responsibility |
|------|----------------|
| `crates/planning-core/src/private/lifecycle.rs` | `Lifecycle`, `Completion`, `Achievement` |
| `crates/planning-core/src/private/classification.rs` | `Classification` |
| `crates/planning-core/src/private/value.rs` | `Value` + `CreateValue` |
| `crates/planning-core/src/private/goal.rs` | `Goal` + `CreateGoal` |
| `crates/planning-core/src/private/task.rs` | `Task` + `CreateTask` |
| `crates/planning-core/src/private/habit.rs` | `Habit`, `HabitStrength` + `CreateHabit` |
| `crates/planning-core/src/private/cadence.rs` | `Cadence`, `WeekdaySet` |
| `crates/planning-core/src/private/association.rs` | `Association`, `AssociationEnd`, pair validation |
| `crates/planning-core/src/private/domain_error.rs` | `DomainError` |
| `crates/planning-store/src/private/records.rs` | Generic `Records` gateway |
| `crates/planning-app/src/private/library.rs` | Creation use cases |
| `crates/planning-app/src/private/entity_lifecycle.rs` | Archive, restore, complete, reopen, achieve |
| `crates/planning-app/src/private/associations.rs` | Link and unlink use cases |
| `crates/planning-app/src/private/views.rs` | Read models: `TaskView`, `LibraryView`, … |
| `src-tauri/src/private/library_commands.rs` | Tauri commands |
| `src/lib/domain/index.ts` | TypeScript mirror of the read models |
| `src/lib/api/index.ts` | Library API functions |
| `docs/architecture/planning-domain.md` | New architecture doc |
| `docs/flows/archiving-an-entity.md` | New flow doc |

---

### Task 1: Lifecycle vocabulary and classification

**Files:**
- Create: `crates/planning-core/src/private/lifecycle.rs`,
  `crates/planning-core/src/private/classification.rs`,
  `crates/planning-core/src/private/domain_error.rs`
- Modify: `crates/planning-core/src/private/mod.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `Lifecycle { Active, Archived }` with `is_active() -> bool`
  - `Completion { Open, Completed { on: NaiveDate } }` with `is_complete() -> bool`
  - `Achievement { Pursuing, Achieved { on: NaiveDate } }` with `is_achieved() -> bool`
  - `Classification { Unclassified, Low, High }` — `Default` is `Unclassified`
  - `DomainError` with variants `BlankTitle`, `UnsupportedAssociation { left, right }`,
    `EmptyCadence`
  - All serde as `camelCase`-tagged enums so the TypeScript mirror is mechanical.

- [ ] **Step 1: Write the failing test**

`crates/planning-core/src/private/lifecycle.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    #[test]
    fn lifecycle_and_completion_are_independent_axes() {
        // An archived Task remembers that it was completed, so restoring it does
        // not have to guess. This is why they are not one enum.
        let completed = Completion::Completed { on: date() };
        assert!(completed.is_complete());
        assert!(!Lifecycle::Archived.is_active());
        assert!(Lifecycle::Active.is_active());
    }

    #[test]
    fn serde_tags_are_camel_case_for_the_frontend_mirror() {
        assert_eq!(serde_json::to_string(&Lifecycle::Archived).unwrap(), r#""archived""#);
        assert_eq!(
            serde_json::to_string(&Completion::Completed { on: date() }).unwrap(),
            r#"{"status":"completed","on":"2026-08-07"}"#
        );
        assert_eq!(serde_json::to_string(&Completion::Open).unwrap(), r#"{"status":"open"}"#);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
cargo test -p planning-core lifecycle
```

Expected: FAIL — `cannot find type 'Lifecycle'`.

- [ ] **Step 3: Implement `lifecycle.rs`**

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Whether an entity is in everyday use or resting in the Archive. Archiving is
/// always reversible and never deletes (ADR 0002).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Lifecycle {
    #[default]
    Active,
    Archived,
}

impl Lifecycle {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// A Task's outcome. Orthogonal to `Lifecycle`: archiving a completed Task keeps
/// the completion, so restoring it returns it exactly as it was.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Completion {
    #[default]
    Open,
    Completed { on: NaiveDate },
}

impl Completion {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}

/// A Goal's outcome. Same orthogonality as `Completion`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Achievement {
    #[default]
    Pursuing,
    Achieved { on: NaiveDate },
}

impl Achievement {
    pub fn is_achieved(&self) -> bool {
        matches!(self, Self::Achieved { .. })
    }
}
```

`classification.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Manual assessment of Importance or Urgency. Unclassified is a real answer,
/// not a missing value — creating a Task from a title alone must be frictionless.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Classification {
    #[default]
    Unclassified,
    Low,
    High,
}
```

`domain_error.rs`:

```rust
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("a title cannot be blank")]
    BlankTitle,

    #[error("{left} cannot be associated with {right}")]
    UnsupportedAssociation { left: &'static str, right: &'static str },

    #[error("a habit cadence must include at least one weekday")]
    EmptyCadence,
}
```

- [ ] **Step 4: Run it to verify it passes, export, and commit**

Add the three modules to `private/mod.rs` and re-export
`Achievement, Classification, Completion, DomainError, Lifecycle` from `lib.rs`. Add `serde_json`
to `planning-core`'s dev-dependencies if plan 0003 did not already.

```bash
cargo test -p planning-core lifecycle
```

Expected: PASS — 2 tests.

```bash
git add crates/planning-core
git commit -m "feat(core): add orthogonal lifecycle, completion, and classification"
```

---

### Task 2: Cadence

**Files:**
- Create: `crates/planning-core/src/private/cadence.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: `DomainError`.
- Produces:
  - `WeekdaySet` — `from_weekdays(&[Weekday]) -> Self`, `contains(Weekday) -> bool`,
    `is_empty() -> bool`, `weekdays() -> Vec<Weekday>`. Serializes as an array of lowercase
    English day names (`["mon","wed"]`) so report front matter and the UI stay readable.
  - `Cadence { EveryDay, OnWeekdays(WeekdaySet) }` — `new_on_weekdays(&[Weekday]) -> Result<Self, DomainError>`,
    `is_due(Weekday) -> bool`.

Plan 0005 calls `Cadence::is_due` to decide whether a pinned Habit joins a Daily Plan.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_day_is_due_on_every_weekday() {
        for day in [Weekday::Mon, Weekday::Sat, Weekday::Sun] {
            assert!(Cadence::EveryDay.is_due(day));
        }
    }

    #[test]
    fn selected_weekdays_are_due_only_on_those_days() {
        let cadence = Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Wed]).unwrap();
        assert!(cadence.is_due(Weekday::Mon));
        assert!(cadence.is_due(Weekday::Wed));
        assert!(!cadence.is_due(Weekday::Tue));
        assert!(!cadence.is_due(Weekday::Sun));
    }

    #[test]
    fn an_empty_cadence_is_rejected_at_construction() {
        assert_eq!(Cadence::new_on_weekdays(&[]), Err(DomainError::EmptyCadence));
    }

    #[test]
    fn duplicate_weekdays_collapse() {
        let cadence = Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Mon]).unwrap();
        let Cadence::OnWeekdays(days) = cadence else { panic!("expected OnWeekdays") };
        assert_eq!(days.weekdays(), vec![Weekday::Mon]);
    }

    #[test]
    fn weekday_sets_serialize_as_readable_day_names() {
        let days = WeekdaySet::from_weekdays(&[Weekday::Wed, Weekday::Mon]);
        assert_eq!(serde_json::to_string(&days).unwrap(), r#"["mon","wed"]"#);
        let parsed: WeekdaySet = serde_json::from_str(r#"["mon","wed"]"#).unwrap();
        assert_eq!(parsed, days);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
cargo test -p planning-core cadence
```

Expected: FAIL — `cannot find type 'Cadence'`.

- [ ] **Step 3: Implement `cadence.rs`**

```rust
use super::domain_error::DomainError;
use chrono::Weekday;
use serde::{Deserialize, Serialize};

const ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

const NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// A set of weekdays stored as a 7-bit mask but serialized as readable names,
/// because these values end up in Weekly Report front matter that humans edit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct WeekdaySet(u8);

impl WeekdaySet {
    pub fn from_weekdays(days: &[Weekday]) -> Self {
        let mut mask = 0u8;
        for day in days {
            mask |= 1 << position(*day);
        }
        Self(mask)
    }

    pub fn contains(&self, day: Weekday) -> bool {
        self.0 & (1 << position(day)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Always in Monday-first order regardless of insertion order.
    pub fn weekdays(&self) -> Vec<Weekday> {
        ORDER.into_iter().filter(|day| self.contains(*day)).collect()
    }
}

fn position(day: Weekday) -> usize {
    day.num_days_from_monday() as usize
}

impl From<WeekdaySet> for Vec<String> {
    fn from(set: WeekdaySet) -> Self {
        set.weekdays().into_iter().map(|day| NAMES[position(day)].to_string()).collect()
    }
}

impl TryFrom<Vec<String>> for WeekdaySet {
    type Error = DomainError;

    fn try_from(names: Vec<String>) -> Result<Self, Self::Error> {
        let mut days = Vec::new();
        for name in names {
            let index = NAMES
                .iter()
                .position(|candidate| *candidate == name)
                .ok_or(DomainError::EmptyCadence)?;
            days.push(ORDER[index]);
        }
        Ok(Self::from_weekdays(&days))
    }
}

/// The days on which a Habit is due. Changes apply from the next Daily Plan only
/// (ADR 0002).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Cadence {
    EveryDay,
    OnWeekdays { days: WeekdaySet },
}

impl Cadence {
    pub fn new_on_weekdays(days: &[Weekday]) -> Result<Self, DomainError> {
        let set = WeekdaySet::from_weekdays(days);
        if set.is_empty() {
            return Err(DomainError::EmptyCadence);
        }
        Ok(Self::OnWeekdays { days: set })
    }

    pub fn is_due(&self, day: Weekday) -> bool {
        match self {
            Self::EveryDay => true,
            Self::OnWeekdays { days } => days.contains(day),
        }
    }
}
```

The test destructures `Cadence::OnWeekdays(days)`; with the struct variant above it must read
`Cadence::OnWeekdays { days }`. Fix the test to match — the struct variant is what gives the
readable `{"kind":"onWeekdays","days":["mon"]}` JSON the UI wants.

- [ ] **Step 4: Run, export, commit**

```bash
cargo test -p planning-core cadence
```

Expected: PASS — 5 tests.

```bash
git add crates/planning-core
git commit -m "feat(core): add habit cadence with a readable weekday set"
```

---

### Task 3: The four entities

**Files:**
- Create: `crates/planning-core/src/private/value.rs`, `goal.rs`, `task.rs`, `habit.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: ids, `Clock`, `Lifecycle`, `Completion`, `Achievement`, `Classification`, `Cadence`,
  `DomainError`.
- Produces (fields are all `pub`; these are data, not encapsulated objects):

```rust
pub struct Value  { pub id: ValueId, pub title: String, pub lifecycle: Lifecycle, pub created_at: DateTime<Utc> }
pub struct Goal   { pub id: GoalId, pub title: String, pub achievement: Achievement,
                    pub lifecycle: Lifecycle, pub target_date: Option<NaiveDate>, pub created_at: DateTime<Utc> }
pub struct Task   { pub id: TaskId, pub title: String, pub completion: Completion,
                    pub lifecycle: Lifecycle, pub importance: Classification, pub urgency: Classification,
                    pub deadline: Option<NaiveDate>, pub created_at: DateTime<Utc> }
pub struct Habit  { pub id: HabitId, pub title: String, pub lifecycle: Lifecycle,
                    pub strength: HabitStrength, pub cadence: Cadence, pub pinned: bool,
                    pub created_at: DateTime<Utc> }
pub enum HabitStrength { ReminderDependent, CueTriggered, Strengthening, Established }
```

Constructors, all taking a single request struct because of the 3-parameter rule:

```rust
Value::create(CreateValue { title: String, clock: &dyn Clock }) -> Result<Value, DomainError>
Goal::create(CreateGoal { title: String, target_date: Option<NaiveDate>, clock: &dyn Clock }) -> Result<Goal, DomainError>
Task::create(CreateTask { title: String, clock: &dyn Clock }) -> Result<Task, DomainError>
Habit::create(CreateHabit { title: String, cadence: Cadence, clock: &dyn Clock }) -> Result<Habit, DomainError>
```

Also `Task::is_overdue(&self, today: NaiveDate) -> bool`.

- [ ] **Step 1: Write the failing creation-rule tests**

Put these in `task.rs` and `habit.rs`; the `Value`/`Goal` cases go in their own files following the
same shape.

`task.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, day).unwrap()
    }

    #[test]
    fn a_title_alone_is_enough_to_create_a_task() {
        let task = Task::create(CreateTask { title: "Draft the letter".into(), clock: &clock() })
            .unwrap();
        assert_eq!(task.title, "Draft the letter");
        assert_eq!(task.completion, Completion::Open);
        assert_eq!(task.lifecycle, Lifecycle::Active);
        assert_eq!(task.importance, Classification::Unclassified);
        assert_eq!(task.urgency, Classification::Unclassified);
        assert_eq!(task.deadline, None);
    }

    #[test]
    fn blank_titles_are_rejected_and_whitespace_is_trimmed() {
        assert_eq!(
            Task::create(CreateTask { title: "   ".into(), clock: &clock() }),
            Err(DomainError::BlankTitle)
        );
        let task = Task::create(CreateTask { title: "  Tidy  ".into(), clock: &clock() }).unwrap();
        assert_eq!(task.title, "Tidy");
    }

    #[test]
    fn a_task_is_overdue_only_while_it_is_open_active_and_past_its_deadline() {
        let mut task = Task::create(CreateTask { title: "File taxes".into(), clock: &clock() })
            .unwrap();
        assert!(!task.is_overdue(day(7)), "no deadline is never overdue");

        task.deadline = Some(day(6));
        assert!(task.is_overdue(day(7)));
        assert!(!task.is_overdue(day(6)), "the deadline day itself is not yet overdue");

        task.completion = Completion::Completed { on: day(7) };
        assert!(!task.is_overdue(day(7)), "completed work is not overdue");

        task.completion = Completion::Open;
        task.lifecycle = Lifecycle::Archived;
        assert!(!task.is_overdue(day(7)), "archived work is not actionable, so not overdue");
    }
}
```

`habit.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::{TimeZone, Weekday};

    #[test]
    fn a_new_habit_is_pinned_and_reminder_dependent() {
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());
        let habit = Habit::create(CreateHabit {
            title: "Writing practice".into(),
            cadence: Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Thu]).unwrap(),
            clock: &clock,
        })
        .unwrap();

        assert!(habit.pinned, "new habits are pinned by default");
        assert_eq!(habit.strength, HabitStrength::ReminderDependent);
        assert_eq!(habit.lifecycle, Lifecycle::Active);
        assert!(habit.cadence.is_due(Weekday::Thu));
    }
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p planning-core
```

Expected: FAIL — `cannot find struct 'Task'` / `'Habit'`.

- [ ] **Step 3: Implement `task.rs`**

```rust
use super::classification::Classification;
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::TaskId;
use super::lifecycle::{Completion, Lifecycle};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateTask<'a> {
    pub title: String,
    pub clock: &'a dyn Clock,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub completion: Completion,
    pub lifecycle: Lifecycle,
    pub importance: Classification,
    pub urgency: Classification,
    pub deadline: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// A title alone is enough — everything else defaults. Friction here would
    /// push capture out of the app.
    pub fn create(request: CreateTask<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: TaskId::generate(),
            title: clean_title(request.title)?,
            completion: Completion::Open,
            lifecycle: Lifecycle::Active,
            importance: Classification::Unclassified,
            urgency: Classification::Unclassified,
            deadline: None,
            created_at: request.clock.now(),
        })
    }

    /// A missed Deadline makes a Task Overdue without changing it — this is a
    /// projection, never stored (CONTEXT.md).
    pub fn is_overdue(&self, today: NaiveDate) -> bool {
        if !self.lifecycle.is_active() || self.completion.is_complete() {
            return false;
        }
        self.deadline.is_some_and(|deadline| deadline < today)
    }
}

pub(crate) fn clean_title(title: String) -> Result<String, DomainError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankTitle);
    }
    Ok(trimmed.to_string())
}
```

- [ ] **Step 4: Implement `value.rs`, `goal.rs`, and `habit.rs`**

`value.rs`:

```rust
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::ValueId;
use super::lifecycle::Lifecycle;
use super::task::clean_title;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateValue<'a> {
    pub title: String,
    pub clock: &'a dyn Clock,
}

/// An enduring personal principle. Values are active or archived — never completed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub id: ValueId,
    pub title: String,
    pub lifecycle: Lifecycle,
    pub created_at: DateTime<Utc>,
}

impl Value {
    pub fn create(request: CreateValue<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: ValueId::generate(),
            title: clean_title(request.title)?,
            lifecycle: Lifecycle::Active,
            created_at: request.clock.now(),
        })
    }
}
```

`goal.rs`:

```rust
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::GoalId;
use super::lifecycle::{Achievement, Lifecycle};
use super::task::clean_title;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateGoal<'a> {
    pub title: String,
    pub target_date: Option<NaiveDate>,
    pub clock: &'a dyn Clock,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub achievement: Achievement,
    pub lifecycle: Lifecycle,
    pub target_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl Goal {
    pub fn create(request: CreateGoal<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: GoalId::generate(),
            title: clean_title(request.title)?,
            achievement: Achievement::Pursuing,
            lifecycle: Lifecycle::Active,
            target_date: request.target_date,
            created_at: request.clock.now(),
        })
    }
}
```

`habit.rs`:

```rust
use super::cadence::Cadence;
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::HabitId;
use super::lifecycle::Lifecycle;
use super::task::clean_title;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The qualitative stage of a Habit. Manual — the app never scores it (PRODUCT.md:
/// reflect, don't score).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HabitStrength {
    #[default]
    ReminderDependent,
    CueTriggered,
    Strengthening,
    Established,
}

pub struct CreateHabit<'a> {
    pub title: String,
    pub cadence: Cadence,
    pub clock: &'a dyn Clock,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: HabitId,
    pub title: String,
    pub lifecycle: Lifecycle,
    pub strength: HabitStrength,
    pub cadence: Cadence,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
}

impl Habit {
    /// Unlike a Task, a Habit cannot be created from a title alone — a cadence is
    /// required, because a Habit with no due days would never appear anywhere.
    pub fn create(request: CreateHabit<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: HabitId::generate(),
            title: clean_title(request.title)?,
            lifecycle: Lifecycle::Active,
            strength: HabitStrength::ReminderDependent,
            cadence: request.cadence,
            pinned: true,
            created_at: request.clock.now(),
        })
    }
}
```

- [ ] **Step 5: Add the `Value` and `Goal` tests**

Mirror the Task tests: blank title rejected, whitespace trimmed, `Lifecycle::Active` and
`Achievement::Pursuing` on creation, `target_date` passed through as given (including `None`).

- [ ] **Step 6: Run, export, commit**

```bash
cargo test -p planning-core && cargo clippy -p planning-core --all-targets -- -D warnings
```

Expected: PASS.

```bash
git add crates/planning-core
git commit -m "feat(core): add Value, Goal, Task, and Habit entities with creation rules"
```

---

### Task 4: Associations

**Files:**
- Create: `crates/planning-core/src/private/association.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: ids, `Clock`, `DomainError`.
- Produces:
  - `AssociationEnd { Value(ValueId), Goal(GoalId), Habit(HabitId), Task(TaskId) }` with
    `kind() -> &'static str` and `rank() -> u8`.
  - `Association { id, left: AssociationEnd, right: AssociationEnd, created_at }`
  - `Association::link(Link { left, right, clock }) -> Result<Association, DomainError>` —
    validates the pair and stores it in canonical order.
  - `Association::touches(&self, &AssociationEnd) -> bool`
  - `Association::other_side(&self, &AssociationEnd) -> Option<&AssociationEnd>`

Supported pairs, per `CONTEXT.md`: Value–Goal, Goal–Habit, Task–Goal, Task–Habit. Everything else
is `DomainError::UnsupportedAssociation`. Canonical ordering (Value < Goal < Habit < Task) makes
"is this link already present?" a plain equality check instead of a two-way search.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn goal() -> AssociationEnd {
        AssociationEnd::Goal(GoalId::new("g1"))
    }

    fn value() -> AssociationEnd {
        AssociationEnd::Value(ValueId::new("v1"))
    }

    #[test]
    fn supported_pairs_link_in_either_direction_and_normalize() {
        let forward =
            Association::link(Link { left: value(), right: goal(), clock: &clock() }).unwrap();
        let backward =
            Association::link(Link { left: goal(), right: value(), clock: &clock() }).unwrap();

        assert_eq!(forward.left, value(), "Value sorts before Goal");
        assert_eq!(forward.right, goal());
        assert_eq!(
            (backward.left.clone(), backward.right.clone()),
            (forward.left.clone(), forward.right.clone()),
            "direction must not create a second distinct link"
        );
    }

    #[test]
    fn unsupported_pairs_are_rejected() {
        let error = Association::link(Link {
            left: value(),
            right: AssociationEnd::Task(TaskId::new("t1")),
            clock: &clock(),
        })
        .unwrap_err();
        assert_eq!(
            error,
            DomainError::UnsupportedAssociation { left: "value", right: "task" }
        );

        assert!(Association::link(Link { left: goal(), right: goal(), clock: &clock() }).is_err());
    }

    #[test]
    fn a_link_can_report_the_other_side() {
        let link = Association::link(Link { left: value(), right: goal(), clock: &clock() }).unwrap();
        assert!(link.touches(&goal()));
        assert_eq!(link.other_side(&goal()), Some(&value()));
        assert_eq!(link.other_side(&AssociationEnd::Task(TaskId::new("t9"))), None);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
cargo test -p planning-core association
```

Expected: FAIL — `cannot find struct 'Association'`.

- [ ] **Step 3: Implement `association.rs`**

```rust
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::{AssociationId, GoalId, HabitId, TaskId, ValueId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum AssociationEnd {
    Value(ValueId),
    Goal(GoalId),
    Habit(HabitId),
    Task(TaskId),
}

impl AssociationEnd {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "value",
            Self::Goal(_) => "goal",
            Self::Habit(_) => "habit",
            Self::Task(_) => "task",
        }
    }

    /// Canonical sort order. Storing every link in one direction turns duplicate
    /// detection into equality instead of a two-way search.
    fn rank(&self) -> u8 {
        match self {
            Self::Value(_) => 0,
            Self::Goal(_) => 1,
            Self::Habit(_) => 2,
            Self::Task(_) => 3,
        }
    }
}

pub struct Link<'a> {
    pub left: AssociationEnd,
    pub right: AssociationEnd,
    pub clock: &'a dyn Clock,
}

/// A many-to-many relevance link. Never implies ownership: archiving one side
/// leaves the other untouched and keeps the link dormant (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Association {
    pub id: AssociationId,
    pub left: AssociationEnd,
    pub right: AssociationEnd,
    pub created_at: DateTime<Utc>,
}

/// The only pairs CONTEXT.md defines, expressed once so the rule has one home.
const SUPPORTED: [(&str, &str); 4] =
    [("value", "goal"), ("goal", "habit"), ("goal", "task"), ("habit", "task")];

impl Association {
    pub fn link(request: Link<'_>) -> Result<Self, DomainError> {
        let (left, right) = canonical(request.left, request.right);
        let pair = (left.kind(), right.kind());
        if !SUPPORTED.contains(&pair) {
            return Err(DomainError::UnsupportedAssociation { left: pair.0, right: pair.1 });
        }
        Ok(Self { id: AssociationId::generate(), left, right, created_at: request.clock.now() })
    }

    pub fn touches(&self, end: &AssociationEnd) -> bool {
        self.left == *end || self.right == *end
    }

    pub fn other_side(&self, end: &AssociationEnd) -> Option<&AssociationEnd> {
        if self.left == *end {
            return Some(&self.right);
        }
        if self.right == *end {
            return Some(&self.left);
        }
        None
    }
}

fn canonical(left: AssociationEnd, right: AssociationEnd) -> (AssociationEnd, AssociationEnd) {
    if left.rank() <= right.rank() {
        return (left, right);
    }
    (right, left)
}
```

A Goal–Goal link canonicalizes to `("goal", "goal")`, which is not in `SUPPORTED`, so self-linking
is rejected by the same rule — no special case needed.

- [ ] **Step 4: Run, export, commit**

```bash
cargo test -p planning-core association
```

Expected: PASS — 3 tests.

```bash
git add crates/planning-core
git commit -m "feat(core): add associations with canonical ordering and pair validation"
```

---

### Task 5: The `Records` gateway

**Files:**
- Create: `crates/planning-store/src/private/records.rs`
- Modify: `crates/planning-store/src/private/mod.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: `Database`, `StoreError`.
- Produces:
  - `RecordKey<'a> { table: &'a str, id: &'a str }`
  - `Records::save<T>(&Database, RecordKey, &T) -> Result<(), StoreError>` — upsert
  - `Records::find<T>(&Database, RecordKey) -> Result<Option<T>, StoreError>`
  - `Records::all<T>(&Database, table: &str) -> Result<Vec<T>, StoreError>`

  where `T: Serialize + DeserializeOwned + Send + Sync + 'static`.

One generic gateway instead of six near-identical repositories: `fallow dupes` would flag the
alternative, and every entity's persistence is genuinely identical because SurrealDB stores
documents. There is deliberately **no `delete`** — ADR 0002.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Note {
        title: String,
    }

    async fn database() -> (TempDir, Database) {
        let folder = TempDir::new().unwrap();
        let database = Database::open(folder.path()).await.unwrap();
        (folder, database)
    }

    #[tokio::test]
    async fn records_round_trip_and_saving_twice_updates_rather_than_duplicates() {
        let (_folder, database) = database().await;
        let key = RecordKey { table: "note", id: "n1" };

        Records::save(&database, key, &Note { title: "first".into() }).await.unwrap();
        Records::save(&database, key, &Note { title: "second".into() }).await.unwrap();

        let found: Option<Note> = Records::find(&database, key).await.unwrap();
        assert_eq!(found, Some(Note { title: "second".into() }));

        let all: Vec<Note> = Records::all(&database, "note").await.unwrap();
        assert_eq!(all.len(), 1, "saving twice must not create a second record");
    }

    #[tokio::test]
    async fn a_missing_record_is_none_rather_than_an_error() {
        let (_folder, database) = database().await;
        let found: Option<Note> =
            Records::find(&database, RecordKey { table: "note", id: "absent" }).await.unwrap();
        assert_eq!(found, None);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
cargo test -p planning-store records
```

Expected: FAIL — `cannot find struct 'Records'`.

- [ ] **Step 3: Implement `records.rs`**

```rust
use super::database::Database;
use super::error::StoreError;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub struct RecordKey<'a> {
    pub table: &'a str,
    pub id: &'a str,
}

/// The single persistence gateway. Every entity stores the same way, so there is
/// one implementation rather than six.
///
/// There is no `delete` and there never will be: nothing is hard-deleted (ADR 0002).
pub struct Records;

impl Records {
    pub async fn save<T>(
        database: &Database,
        key: RecordKey<'_>,
        record: &T,
    ) -> Result<(), StoreError>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let _saved: Option<T> = database
            .inner()
            .upsert((key.table, key.id))
            .content(record)
            .await?;
        Ok(())
    }

    pub async fn find<T>(database: &Database, key: RecordKey<'_>) -> Result<Option<T>, StoreError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        Ok(database.inner().select((key.table, key.id)).await?)
    }

    pub async fn all<T>(database: &Database, table: &str) -> Result<Vec<T>, StoreError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        Ok(database.inner().select(table).await?)
    }
}
```

If `.content(record)` requires an owned value, change the signature to take `record: T` and clone
at the call site. If `upsert` is unavailable, fall back to the `UPSERT ... CONTENT $data` query
form noted in plan 0003 Task 5.

- [ ] **Step 4: Run, export `Records` and `RecordKey` from `lib.rs`, commit**

```bash
cargo test -p planning-store records
```

Expected: PASS — 2 tests.

```bash
git add crates/planning-store
git commit -m "feat(store): add the generic Records gateway with no delete path"
```

---

### Task 6: Library creation use cases

**Files:**
- Create: `crates/planning-app/src/private/library.rs`
- Modify: `crates/planning-app/src/private/mod.rs`, `crates/planning-app/src/lib.rs`,
  `crates/planning-app/Cargo.toml` (add `chrono` if absent)

**Interfaces:**
- Consumes: `PlanningApp::require_database`, `PlanningApp::clock`, entity constructors, `Records`.
- Produces, as methods on `PlanningApp`:
  - `create_value(&self, title: String) -> Result<Value, AppError>`
  - `create_goal(&self, NewGoal { title, target_date }) -> Result<Goal, AppError>`
  - `create_task(&self, title: String) -> Result<Task, AppError>`
  - `create_habit(&self, NewHabit { title, cadence }) -> Result<Habit, AppError>`
  - `AppError::Domain(#[from] DomainError)`

Every one refuses when `StoreHealth` is not `Ready`, because `require_database` returns `Err`.
That is the whole sync-safety enforcement — no separate check to forget.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::service::StartRequest;
    use chrono::{TimeZone, Weekday};
    use planning_core::{Cadence, Classification, FixedClock, Lifecycle};
    use std::sync::Arc;
    use tempfile::TempDir;

    /// A fully set-up app: device settings in one temp dir, sync folder in another.
    async fn ready_app() -> (TempDir, TempDir, PlanningApp) {
        let home = TempDir::new().unwrap();
        let drive = TempDir::new().unwrap();
        let mut app = PlanningApp::start(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())),
        })
        .await
        .unwrap();
        app.choose_sync_folder(drive.path().to_path_buf()).await.unwrap();
        app.set_home_zone(chrono_tz::Tz::Europe__Madrid).await.unwrap();
        (home, drive, app)
    }

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
        let home = TempDir::new().unwrap();
        let app = PlanningApp::start(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())),
        })
        .await
        .unwrap();

        let error = app.create_task("Anything".into()).await.unwrap_err();
        assert!(matches!(error, AppError::NotReady(_) | AppError::NoDatabase));
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
```

- [ ] **Step 2: Run it to verify it fails**

```bash
cargo test -p planning-app library
```

Expected: FAIL — `no method named 'create_task'`.

- [ ] **Step 3: Implement `library.rs`**

```rust
use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{
    Cadence, CreateGoal, CreateHabit, CreateTask, CreateValue, Goal, Habit, Task, Value,
};
use planning_store::{RecordKey, Records};

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
        let value = Value::create(CreateValue { title, clock: self.clock.as_ref() })?;
        self.store(ValueId::TABLE, value.id.as_str(), &value).await?;
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
        let task = Task::create(CreateTask { title, clock: self.clock.as_ref() })?;
        self.store(TaskId::TABLE, task.id.as_str(), &task).await?;
        Ok(task)
    }

    pub async fn create_habit(&self, request: NewHabit) -> Result<Habit, AppError> {
        let habit = Habit::create(CreateHabit {
            title: request.title,
            cadence: request.cadence,
            clock: self.clock.as_ref(),
        })?;
        self.store(HabitId::TABLE, habit.id.as_str(), &habit).await?;
        Ok(habit)
    }
}
```

Add the shared persistence helpers to `service.rs` so every use-case file reuses them (this keeps
`library.rs` under 200 lines and stops the pattern being retyped in Tasks 7 and 8):

```rust
impl PlanningApp {
    /// Saves one record, refusing unless the store is Ready.
    pub(crate) async fn store<T>(&self, table: &str, id: &str, record: &T) -> Result<(), AppError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Records::save(database, RecordKey { table, id }, record).await?;
        Ok(())
    }

    pub(crate) async fn load_all<T>(&self, table: &str) -> Result<Vec<T>, AppError>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Ok(Records::all(database, table).await?)
    }

    pub(crate) async fn load_one<T>(&self, table: &str, id: &str) -> Result<Option<T>, AppError>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Ok(Records::find(database, RecordKey { table, id }).await?)
    }
}
```

Add the raw accessors used by the tests and by Task 7:

```rust
impl PlanningApp {
    pub async fn values(&self) -> Result<Vec<Value>, AppError> { self.load_all(ValueId::TABLE).await }
    pub async fn goals(&self) -> Result<Vec<Goal>, AppError> { self.load_all(GoalId::TABLE).await }
    pub async fn tasks(&self) -> Result<Vec<Task>, AppError> { self.load_all(TaskId::TABLE).await }
    pub async fn habits(&self) -> Result<Vec<Habit>, AppError> { self.load_all(HabitId::TABLE).await }
}
```

Add `#[error(transparent)] Domain(#[from] DomainError)` to `AppError`, and re-export the entity
types and `NewGoal`/`NewHabit` from `planning-app`'s `lib.rs`.

- [ ] **Step 4: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS.

```bash
git add crates/planning-app
git commit -m "feat(app): add Library creation use cases gated on store readiness"
```

---

### Task 7: Reversible lifecycle — archive, restore, complete, reopen, achieve

**Files:**
- Create: `crates/planning-app/src/private/entity_lifecycle.rs`
- Modify: `crates/planning-app/src/private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: the load/store helpers from Task 6, `HomeCalendar` from plan 0003.
- Produces, as methods on `PlanningApp`:
  - `archive_task(&self, &TaskId)`, `restore_task(&self, &TaskId)`
  - `archive_habit`, `restore_habit`, `archive_goal`, `restore_goal`, `archive_value`, `restore_value`
  - `complete_task(&self, &TaskId)`, `reopen_task(&self, &TaskId)`
  - `achieve_goal(&self, &GoalId)`, `unachieve_goal(&self, &GoalId)`
  - `set_task_classification(&self, ClassifyTask { task, importance, urgency })`
  - `set_task_deadline(&self, SetDeadline { task, deadline })`
  - `set_habit_cadence(&self, SetCadence { habit, cadence })`
  - `set_habit_pinned(&self, SetPinned { habit, pinned })`
  - `set_habit_strength(&self, SetStrength { habit, strength })`
  - `AppError::NotFound { table: &'static str, id: String }`

  All return `Result<(), AppError>` except where noted. Completion dates come from
  `self.calendar()?.today(self.clock.as_ref())` — never from the device zone.

**This task claims acceptance criterion A6's first half**: archiving marks the entity and blocks
future selection. Plan 0005 claims the second half — that an existing Daily Plan entry survives and
stays completable.

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // `ready_app` is duplicated from library.rs's tests; extract it into
    // crates/planning-app/src/private/test_support.rs behind #[cfg(test)] and
    // import it from both files rather than copying it a third time.
    use crate::private::test_support::ready_app;
    use planning_core::{Achievement, Completion, Lifecycle};

    #[tokio::test]
    async fn archiving_is_reversible_and_preserves_completion() {
        let (_home, _drive, app) = ready_app().await;
        let task = app.create_task("File taxes".into()).await.unwrap();

        app.complete_task(&task.id).await.unwrap();
        app.archive_task(&task.id).await.unwrap();

        let archived = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(archived.lifecycle, Lifecycle::Archived);
        assert!(archived.completion.is_complete(), "archiving must not erase the outcome");

        app.restore_task(&task.id).await.unwrap();
        let restored = app.task(&task.id).await.unwrap().unwrap();
        assert_eq!(restored.lifecycle, Lifecycle::Active);
        assert!(restored.completion.is_complete(), "restoring returns it exactly as it was");
    }

    #[tokio::test]
    async fn completion_is_reversible_and_dated_in_the_home_zone() {
        let (_home, _drive, app) = ready_app().await;
        let task = app.create_task("Draft the letter".into()).await.unwrap();

        app.complete_task(&task.id).await.unwrap();
        let completed = app.task(&task.id).await.unwrap().unwrap();
        // The fixed clock is 2026-08-07 09:00 UTC; Madrid is UTC+2, still the 7th.
        assert_eq!(
            completed.completion,
            Completion::Completed { on: NaiveDate::from_ymd_opt(2026, 8, 7).unwrap() }
        );

        app.reopen_task(&task.id).await.unwrap();
        assert_eq!(app.task(&task.id).await.unwrap().unwrap().completion, Completion::Open);
    }

    #[tokio::test]
    async fn goals_are_achievable_and_un_achievable() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app
            .create_goal(NewGoal { title: "Career transition".into(), target_date: None })
            .await
            .unwrap();

        app.achieve_goal(&goal.id).await.unwrap();
        assert!(app.goal(&goal.id).await.unwrap().unwrap().achievement.is_achieved());

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
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p planning-app entity_lifecycle
```

Expected: FAIL — `no method named 'archive_task'`.

- [ ] **Step 3: Implement the shared mutate helper in `service.rs`**

Writing eleven near-identical load-mutate-save methods by hand would duplicate the same six lines
eleven times. One helper removes that:

```rust
impl PlanningApp {
    /// Loads a record, applies `change`, and saves it back. The single
    /// read-modify-write path, so "not found" is handled in exactly one place.
    pub(crate) async fn mutate<T>(
        &self,
        key: (&'static str, String),
        change: impl FnOnce(&mut T),
    ) -> Result<T, AppError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let (table, id) = key;
        let mut record: T = self
            .load_one(table, &id)
            .await?
            .ok_or(AppError::NotFound { table, id: id.clone() })?;
        change(&mut record);
        self.store(table, &id, &record).await?;
        Ok(record)
    }
}
```

- [ ] **Step 4: Implement `entity_lifecycle.rs`**

```rust
use super::error::AppError;
use super::service::PlanningApp;
use chrono::{NaiveDate, Weekday};
use planning_core::{
    Achievement, Cadence, Classification, Completion, Goal, GoalId, Habit, HabitId, HabitStrength,
    Lifecycle, Task, TaskId,
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

pub struct SetCadence<'a> {
    pub habit: &'a HabitId,
    pub cadence: Cadence,
}

pub struct SetPinned<'a> {
    pub habit: &'a HabitId,
    pub pinned: bool,
}

pub struct SetStrength<'a> {
    pub habit: &'a HabitId,
    pub strength: HabitStrength,
}

impl PlanningApp {
    fn today(&self) -> Result<NaiveDate, AppError> {
        Ok(self.calendar()?.today(self.clock.as_ref()))
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
        self.mutate::<Task>((TaskId::TABLE, task.to_string()), |found| found.lifecycle = to)
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

    /// Cadence changes apply from the next Daily Plan — nothing here rewrites an
    /// existing plan (ADR 0002).
    pub async fn set_habit_cadence(&self, request: SetCadence<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.cadence = request.cadence;
        })
        .await?;
        Ok(())
    }

    pub async fn set_habit_pinned(&self, request: SetPinned<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.pinned = request.pinned;
        })
        .await?;
        Ok(())
    }

    pub async fn set_habit_strength(&self, request: SetStrength<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.strength = request.strength;
        })
        .await?;
        Ok(())
    }
}
```

Add the remaining archive/restore pairs for `Habit`, `Goal`, and `Value` following the
`set_task_lifecycle` shape, and the single-entity accessors `task`, `goal`, `habit`, `value`
(each `self.load_one(TABLE, id.as_str())`). Add to `AppError`:

```rust
    #[error("no {table} with id {id}")]
    NotFound { table: &'static str, id: String },
```

`entity_lifecycle.rs` will exceed 200 lines with all four entity families. Split it into
`entity_lifecycle.rs` (Task and Goal) and `habit_lifecycle.rs` (Habit and Value) — both `impl
PlanningApp`, both re-exported from `mod.rs`.

- [ ] **Step 5: Run, commit**

```bash
cargo test -p planning-app && cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS.

```bash
git add crates/planning-app
git commit -m "feat(app): add reversible archive, completion, and achievement"
```

---

### Task 8: Association use cases

**Files:**
- Create: `crates/planning-app/src/private/associations.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: `Association`, `Records`.
- Produces:
  - `link(&self, LinkEnds { left, right }) -> Result<Association, AppError>` — idempotent: linking
    an existing pair returns the existing `Association` rather than a duplicate.
  - `unlink(&self, &AssociationId) -> Result<(), AppError>` — **archives** the link by setting
    `Lifecycle::Archived` on it; there is no delete.
  - `associations_for(&self, &AssociationEnd) -> Result<Vec<Association>, AppError>` — active links
    only.
  - `Association` gains `pub lifecycle: Lifecycle` (add it in `planning-core`, defaulting to
    `Active`, and extend that crate's tests).

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app;
    use planning_core::AssociationEnd;

    #[tokio::test]
    async fn linking_the_same_pair_twice_returns_the_same_link() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app.create_goal(NewGoal { title: "Career".into(), target_date: None }).await.unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();

        let ends = || LinkEnds {
            left: AssociationEnd::Task(task.id.clone()),
            right: AssociationEnd::Goal(goal.id.clone()),
        };

        let first = app.link(ends()).await.unwrap();
        let second = app.link(ends()).await.unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(app.associations_for(&AssociationEnd::Goal(goal.id.clone())).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn archiving_one_side_never_cascades_and_the_link_returns_on_restore() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app.create_goal(NewGoal { title: "Career".into(), target_date: None }).await.unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.link(LinkEnds {
            left: AssociationEnd::Task(task.id.clone()),
            right: AssociationEnd::Goal(goal.id.clone()),
        })
        .await
        .unwrap();

        app.archive_task(&task.id).await.unwrap();

        // The Goal is untouched and the link still exists, dormant.
        assert_eq!(app.goal(&goal.id).await.unwrap().unwrap().lifecycle, Lifecycle::Active);
        assert_eq!(
            app.associations_for(&AssociationEnd::Goal(goal.id.clone())).await.unwrap().len(),
            1,
            "the link is preserved, not deleted"
        );

        app.restore_task(&task.id).await.unwrap();
        assert_eq!(
            app.associations_for(&AssociationEnd::Task(task.id.clone())).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn unlinking_archives_the_link_rather_than_deleting_it() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app.create_goal(NewGoal { title: "Career".into(), target_date: None }).await.unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        let link = app
            .link(LinkEnds {
                left: AssociationEnd::Task(task.id.clone()),
                right: AssociationEnd::Goal(goal.id.clone()),
            })
            .await
            .unwrap();

        app.unlink(&link.id).await.unwrap();
        assert!(app.associations_for(&AssociationEnd::Goal(goal.id)).await.unwrap().is_empty());

        let all: Vec<planning_core::Association> = app.all_associations().await.unwrap();
        assert_eq!(all.len(), 1, "the record still exists, archived");
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-app associations
```

Expected: FAIL — `no method named 'link'`.

```rust
use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{Association, AssociationEnd, AssociationId, Lifecycle, Link};

pub struct LinkEnds {
    pub left: AssociationEnd,
    pub right: AssociationEnd,
}

impl PlanningApp {
    /// Idempotent: the canonical ordering in `Association::link` means an existing
    /// pair compares equal regardless of the direction the caller supplied.
    pub async fn link(&self, ends: LinkEnds) -> Result<Association, AppError> {
        let candidate = Association::link(Link {
            left: ends.left,
            right: ends.right,
            clock: self.clock.as_ref(),
        })?;

        let existing = self.all_associations().await?.into_iter().find(|found| {
            found.left == candidate.left && found.right == candidate.right
        });

        if let Some(mut found) = existing {
            if found.lifecycle.is_active() {
                return Ok(found);
            }
            // Re-linking a previously unlinked pair revives it rather than
            // accumulating a second record.
            found.lifecycle = Lifecycle::Active;
            self.store(AssociationId::TABLE, found.id.as_str(), &found).await?;
            return Ok(found);
        }

        self.store(AssociationId::TABLE, candidate.id.as_str(), &candidate).await?;
        Ok(candidate)
    }

    /// Archives the link. ADR 0002 has no delete path, not even for links.
    pub async fn unlink(&self, link: &AssociationId) -> Result<(), AppError> {
        self.mutate::<Association>((AssociationId::TABLE, link.to_string()), |found| {
            found.lifecycle = Lifecycle::Archived;
        })
        .await?;
        Ok(())
    }

    pub async fn all_associations(&self) -> Result<Vec<Association>, AppError> {
        self.load_all(AssociationId::TABLE).await
    }

    /// Active links touching `end`. Archiving an entity does not archive its
    /// links, so a dormant link reappears the moment that entity is restored.
    pub async fn associations_for(
        &self,
        end: &AssociationEnd,
    ) -> Result<Vec<Association>, AppError> {
        Ok(self
            .all_associations()
            .await?
            .into_iter()
            .filter(|found| found.lifecycle.is_active() && found.touches(end))
            .collect())
    }
}
```

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS.

```bash
git add crates/planning-core crates/planning-app
git commit -m "feat(app): add non-cascading associations with archive-only unlink"
```

---

### Task 9: Read models and the Library view

**Files:**
- Create: `crates/planning-app/src/private/views.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Consumes: entities, `associations_for`, `HomeCalendar`.
- Produces:

```rust
pub enum TaskState { Open, Completed, Archived }   // derived, for display only

pub struct TaskView   { id, title, state, importance, urgency, deadline, overdue, archived }
pub struct HabitView  { id, title, cadence, strength, pinned, archived }
pub struct GoalView   { id, title, achieved, target_date, archived }
pub struct ValueView  { id, title, archived }
pub struct LibraryView { values, goals, habits, tasks }
pub struct LibraryFilter { pub include_archived: bool }

PlanningApp::library(&self, LibraryFilter) -> Result<LibraryView, AppError>
```

All view structs are `#[serde(rename_all = "camelCase")]`. `overdue` and `archived` are
**projected here**, never stored — that is what makes forward-only propagation free for plan 0005.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_app;

    #[tokio::test]
    async fn the_library_hides_archived_entries_by_default_and_can_show_them() {
        let (_home, _drive, app) = ready_app().await;
        let kept = app.create_task("Keep".into()).await.unwrap();
        let gone = app.create_task("Archive me".into()).await.unwrap();
        app.archive_task(&gone.id).await.unwrap();

        let everyday = app.library(LibraryFilter { include_archived: false }).await.unwrap();
        assert_eq!(everyday.tasks.len(), 1);
        assert_eq!(everyday.tasks[0].id, kept.id);

        let with_archive = app.library(LibraryFilter { include_archived: true }).await.unwrap();
        assert_eq!(with_archive.tasks.len(), 2);
        let archived = with_archive.tasks.iter().find(|view| view.id == gone.id).unwrap();
        assert!(archived.archived, "archived entries are shown honestly, not hidden");
        assert_eq!(archived.state, TaskState::Archived);
    }

    #[tokio::test]
    async fn overdue_is_projected_from_the_home_date_not_stored() {
        let (_home, _drive, app) = ready_app().await;
        let task = app.create_task("File taxes".into()).await.unwrap();
        app.set_task_deadline(SetDeadline {
            task: &task.id,
            deadline: Some(NaiveDate::from_ymd_opt(2026, 8, 6).unwrap()),
        })
        .await
        .unwrap();

        // The fixed clock is 2026-08-07 in Madrid, so a deadline of the 6th is past.
        let library = app.library(LibraryFilter { include_archived: false }).await.unwrap();
        assert!(library.tasks[0].overdue);

        // Completing it clears the projection without touching the deadline.
        app.complete_task(&task.id).await.unwrap();
        let after = app.library(LibraryFilter { include_archived: false }).await.unwrap();
        assert!(!after.tasks[0].overdue);
        assert_eq!(after.tasks[0].deadline, Some(NaiveDate::from_ymd_opt(2026, 8, 6).unwrap()));
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement `views.rs`**

```bash
cargo test -p planning-app views
```

Expected: FAIL — `cannot find struct 'LibraryFilter'`.

```rust
use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{
    Cadence, Classification, Goal, GoalId, Habit, HabitId, HabitStrength, Task, TaskId, Value,
    ValueId,
};
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
}

impl PlanningApp {
    pub async fn library(&self, filter: LibraryFilter) -> Result<LibraryView, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let keep = |archived: bool| filter.include_archived || !archived;

        Ok(LibraryView {
            values: self
                .values()
                .await?
                .iter()
                .filter(|found| keep(!found.lifecycle.is_active()))
                .map(ValueView::project)
                .collect(),
            goals: self
                .goals()
                .await?
                .iter()
                .filter(|found| keep(!found.lifecycle.is_active()))
                .map(GoalView::project)
                .collect(),
            habits: self
                .habits()
                .await?
                .iter()
                .filter(|found| keep(!found.lifecycle.is_active()))
                .map(HabitView::project)
                .collect(),
            tasks: self
                .tasks()
                .await?
                .iter()
                .filter(|found| keep(!found.lifecycle.is_active()))
                .map(|found| TaskView::project(found, today))
                .collect(),
        })
    }
}
```

`library()` exceeds 30 lines with all four collections inline. Extract each into a small private
function — `project_values(&self, &LibraryFilter)`, `project_goals(...)`, and so on — so `library`
stays a four-line assembly. Write `ValueView`, `GoalView`, and `HabitView` in a sibling
`views_entities.rs` to keep both files under 200 lines:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueView {
    pub id: ValueId,
    pub title: String,
    pub archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    pub id: GoalId,
    pub title: String,
    pub achieved: bool,
    pub target_date: Option<NaiveDate>,
    pub archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitView {
    pub id: HabitId,
    pub title: String,
    pub cadence: Cadence,
    pub strength: HabitStrength,
    pub pinned: bool,
    pub archived: bool,
}
```

Each gets a `project(&Entity) -> Self` following `TaskView::project`'s shape.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS.

```bash
git add crates/planning-app
git commit -m "feat(app): add Library read models with projected overdue and archived flags"
```

---

### Task 10: Tauri commands and the TypeScript mirror

**Files:**
- Create: `src-tauri/src/private/library_commands.rs`, `src/lib/domain/index.ts`
- Modify: `src-tauri/src/lib.rs`, `src/lib/api/index.ts`
- Test: `src/lib/api/library.test.ts`, a Rust JSON-shape test in `views.rs`

**Interfaces:**
- Consumes: every `PlanningApp` method from Tasks 6–9.
- Produces the command surface plan 0007 builds against:

| Command | Args | Returns |
|---------|------|---------|
| `library` | `includeArchived: bool` | `LibraryView` |
| `create_value` / `create_task` | `title: string` | `Value` / `Task` |
| `create_goal` | `title`, `targetDate?` | `Goal` |
| `create_habit` | `title`, `cadence` | `Habit` |
| `archive_entity` / `restore_entity` | `end: AssociationEnd` | `void` |
| `complete_task` / `reopen_task` | `task: string` | `void` |
| `achieve_goal` / `unachieve_goal` | `goal: string` | `void` |
| `classify_task` | `task`, `importance`, `urgency` | `void` |
| `set_task_deadline` | `task`, `deadline?` | `void` |
| `set_habit_cadence` / `set_habit_pinned` / `set_habit_strength` | `habit`, value | `void` |
| `link` / `unlink` | ends / `association` | `Association` / `void` |
| `associations_for` | `end: AssociationEnd` | `Association[]` |

`archive_entity` takes an `AssociationEnd` rather than four separate commands — the enum already
carries the entity kind, so one command covers all four families without a `match` in TypeScript.

- [ ] **Step 1: Write the failing Rust JSON-shape test**

Put it in `views.rs`. It is the contract the TypeScript mirror is written against; without it the
two drift silently.

```rust
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
    };
    assert_eq!(
        serde_json::to_string(&view).unwrap(),
        r#"{"id":"t1","title":"File taxes","state":"open","importance":"high","urgency":"unclassified","deadline":"2026-08-06","overdue":true,"archived":false}"#
    );
}
```

- [ ] **Step 2: Run it, fix any serde attribute mismatch it reveals, then write the commands**

```bash
cargo test -p planning-app views
```

`src-tauri/src/private/library_commands.rs` — each command is a two-line delegation. Keep the file
under 200 lines by splitting into `library_commands.rs` (reads + creation) and
`lifecycle_commands.rs` (mutations + associations).

```rust
use super::commands::AppState;
use planning_app::{LibraryFilter, LibraryView, NewGoal, Task};

#[tauri::command]
pub async fn library(
    state: tauri::State<'_, AppState>,
    include_archived: bool,
) -> Result<LibraryView, String> {
    state
        .0
        .lock()
        .await
        .library(LibraryFilter { include_archived })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    title: String,
) -> Result<Task, String> {
    state.0.lock().await.create_task(title).await.map_err(|error| error.to_string())
}
```

Tauri converts `snake_case` Rust parameters to `camelCase` on the JavaScript side, so
`include_archived` is passed as `includeArchived`. Register every command in
`generate_handler!`.

- [ ] **Step 3: Write `src/lib/domain/index.ts`**

The mirror. Every type here must match a Rust JSON-shape test.

```ts
export type Classification = 'unclassified' | 'low' | 'high';
export type TaskState = 'open' | 'completed' | 'archived';
export type HabitStrength = 'reminderDependent' | 'cueTriggered' | 'strengthening' | 'established';
export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';

export type Cadence = { kind: 'everyDay' } | { kind: 'onWeekdays'; days: Weekday[] };

export type AssociationEnd =
  | { kind: 'value'; id: string }
  | { kind: 'goal'; id: string }
  | { kind: 'habit'; id: string }
  | { kind: 'task'; id: string };

export interface TaskView {
  id: string;
  title: string;
  state: TaskState;
  importance: Classification;
  urgency: Classification;
  deadline: string | null;
  overdue: boolean;
  archived: boolean;
}

export interface ValueView {
  id: string;
  title: string;
  archived: boolean;
}

export interface GoalView {
  id: string;
  title: string;
  achieved: boolean;
  targetDate: string | null;
  archived: boolean;
}

export interface HabitView {
  id: string;
  title: string;
  cadence: Cadence;
  strength: HabitStrength;
  pinned: boolean;
  archived: boolean;
}

export interface LibraryView {
  values: ValueView[];
  goals: GoalView[];
  habits: HabitView[];
  tasks: TaskView[];
}
```

- [ ] **Step 4: Extend `src/lib/api/index.ts` and test it**

Add one exported function per command, all delegating to `call`. Example:

```ts
import type { AssociationEnd, LibraryView } from '../domain';

export function library(includeArchived: boolean): Promise<LibraryView> {
  return call<LibraryView>('library', { includeArchived });
}

export function archiveEntity(end: AssociationEnd): Promise<void> {
  return call<void>('archive_entity', { end });
}
```

`src/lib/api/library.test.ts` mocks `invoke` and asserts the command name and argument object for
each function, following the `appVersion` test from plan 0002.

- [ ] **Step 5: Run the full gate**

```bash
npm run check && fallow audit
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri src/lib
git commit -m "feat: expose the Library API to the frontend"
```

---

### Task 11: Documentation

**Files:**
- Create: `docs/architecture/planning-domain.md`, `docs/flows/archiving-an-entity.md`,
  `docs/lessons-learned/orthogonal-lifecycle-beats-a-single-state-enum.md`
- Modify: the three README index tables, `docs/live/current-status.md`

- [ ] **Step 1: Write `docs/architecture/planning-domain.md`** (target 90 lines)

Cover: the entity table with every field; the orthogonal `Lifecycle` × `Completion` / `Achievement`
design and why; the creation rules (title alone for Value/Goal/Task, cadence required for Habit);
the four supported Association pairs and canonical ordering; that `overdue`, `archived`, and
`TaskState` are **projections computed at read time**, never columns; that `Records` has no delete;
and the `planning-core` → `planning-store` → `planning-app` direction.

- [ ] **Step 2: Write `docs/flows/archiving-an-entity.md`**

Trigger (user archives a Task from the Library) → Entry point (`archive_entity` command) → Steps
(load → set `Lifecycle::Archived` → save) → Reads → Writes → Side effects (**none** — no cascade to
associations, no rewrite of any Daily Plan or Weekly Focus) → Files to inspect → Common failure
modes. State explicitly what does *not* happen, because that is the surprising part.

- [ ] **Step 3: Write the lessons-learned entry**

Topic: `CONTEXT.md` describes three states, but modelling them as one enum makes restore ambiguous
and forces every caller to re-derive "was this completed before it was archived?". Two orthogonal
fields plus a derived `TaskState` for display keeps the domain honest and the UI simple. The
generalizable lesson: when a glossary lists states that a user would describe as combinable
("archived *and* completed"), they are axes, not variants.

- [ ] **Step 4: Register all three in their index tables, update `current-status.md`, commit**

```bash
git add docs
git commit -m "docs: document the planning domain, archiving flow, and lifecycle lesson"
```

---

## Task 12: Verify the plan's own acceptance

- [ ] `npm run check` and `fallow audit` both pass.
- [ ] **A6 (first half):** archiving a Task sets `Lifecycle::Archived`, the Library hides it unless
      `includeArchived`, and it cannot be selected — verified by the Task 9 tests. The second half
      (existing Daily Plan entries survive) is claimed by plan 0005.
- [ ] Archiving a Goal leaves every linked Task and Habit untouched and preserves the link.
- [ ] Restoring an archived completed Task returns it completed, not open.
- [ ] `associations_for` never returns a link whose `lifecycle` is `Archived`.
- [ ] No `DELETE` appears anywhere in `crates/` — `tests/architecture.test.ts` proves it.
- [ ] Every view type in `src/lib/domain/index.ts` has a matching Rust JSON-shape test.

**Next:** [0005-daily-plan-and-habits.md](0005-daily-plan-and-habits.md) (done).
