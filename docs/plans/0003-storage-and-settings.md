# Storage, Settings & Sync Safety — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](0001-self-planning-app.md) first — its **Global Constraints**
> apply to every task here and are not repeated. Requires
> [0002-app-shell.md](0002-app-shell.md) to be complete.

**Goal:** Make the app able to open a Synchronization Folder safely — embedded SurrealDB on
RocksDB, device-local settings kept out of the sync folder, a synchronized home time zone that
governs every date calculation, and a health gate that refuses writes from stale or conflicted data.

**Architecture:** Three new crates. `planning-core` holds the pure time and identity vocabulary
with no IO. `planning-store` owns the SurrealDB connection, the device settings file, and sync
safety detection. `planning-app` is the narrow application API — the only crate `src-tauri` and
`launcher` are permitted to depend on. The health gate is a value, not an exception: every write
path takes a `Ready` proof it cannot fabricate.

**Tech Stack:** surrealdb 3.2 (`kv-rocksdb`), chrono 0.4 + chrono-tz 0.10, directories 6, uuid 1.24
(v7), serde, thiserror 2, tokio (Tauri's runtime), tempfile 3 for tests.

---

## Global constraints

See [0001-self-planning-app.md](0001-self-planning-app.md#global-constraints). Additionally:

- **No `Utc::now()` outside `system_clock.rs`.** Everything that needs the current instant takes
  `&dyn Clock`. `tests/architecture.test.ts` from plan 0002 enforces this.
- **No `chrono::Local` anywhere.** Enforced by the same test.
- **The device time zone is never used for a date calculation.** It may be shown as a *suggestion*
  during setup and nowhere else — and this plan does not even do that: the home zone starts unset
  and setup is incomplete until the user picks one.
- **Device settings never live inside the Synchronization Folder.**

---

## File structure

| File | Responsibility |
|------|----------------|
| `crates/planning-core/src/lib.rs` | Public interface — `pub use` only |
| `crates/planning-core/src/private/ids.rs` | `define_ids!` macro and every id newtype |
| `crates/planning-core/src/private/clock.rs` | `Clock` trait, `FixedClock` test double |
| `crates/planning-core/src/private/system_clock.rs` | `SystemClock` — the only `Utc::now()` caller |
| `crates/planning-core/src/private/calendar_week.rs` | `CalendarWeek` |
| `crates/planning-core/src/private/home_calendar.rs` | `HomeCalendar` |
| `crates/planning-store/src/lib.rs` | Public interface |
| `crates/planning-store/src/private/database.rs` | Opens SurrealDB on RocksDB |
| `crates/planning-store/src/private/device_settings.rs` | Device-local JSON settings |
| `crates/planning-store/src/private/home_settings.rs` | Synchronized home time zone |
| `crates/planning-store/src/private/writer_lock.rs` | One-active-writer lock file |
| `crates/planning-store/src/private/conflicts.rs` | Google Drive conflict artifact detection |
| `crates/planning-store/src/private/health.rs` | `StoreHealth` and `ReadyStore` |
| `crates/planning-store/src/private/error.rs` | `StoreError` |
| `crates/planning-app/src/lib.rs` | Public interface |
| `crates/planning-app/src/private/service.rs` | `PlanningApp` |
| `crates/planning-app/src/private/setup.rs` | Setup use cases |
| `src-tauri/src/private/commands.rs` | Setup + health commands |
| `src/lib/api/index.ts` | `chooseSyncFolder`, `storeHealth`, `setHomeZone` |
| `docs/architecture/storage.md` | New architecture doc |
| `docs/flows/opening-the-app.md` | New flow doc |

---

### Task 0: Windows build prerequisites for SurrealDB + RocksDB

**Do this before anything else.** Both failures below were reproduced on this machine
(Windows 11, MSVC 14.50, Rust 1.95, surrealdb 3.2.4). Without these two fixes the very first
`cargo check -p planning-store` fails after several minutes of compilation with errors that look
unrelated to anything in this plan.

**Files:**
- Modify: root `Cargo.toml`
- Create: `docs/lessons-learned/surrealdb-rocksdb-windows-build-prerequisites.md`

- [ ] **Step 1: Install LLVM so bindgen can find libclang**

`surrealdb-librocksdb-sys` generates its bindings with `bindgen`, which needs `libclang.dll`.
Without it the build panics with:

```
Unable to find libclang: "couldn't find any valid shared libraries matching:
['clang.dll', 'libclang.dll'], set the `LIBCLANG_PATH` environment variable"
```

Install LLVM and make it discoverable:

```bash
winget install --id LLVM.LLVM -e
```

If `C:\Program Files\LLVM\bin\libclang.dll` exists but the build still fails, set the variable
explicitly (PowerShell, persistent for the user):

```powershell
[Environment]::SetEnvironmentVariable('LIBCLANG_PATH', 'C:\Program Files\LLVM\bin', 'User')
```

Verify before continuing:

```bash
ls "/c/Program Files/LLVM/bin/libclang.dll"
```

- [ ] **Step 2: Avoid the NASM requirement from `aws-lc-sys`**

`surrealdb-core` depends on `jsonwebtoken` 10, which pulls `aws-lc-rs` → `aws-lc-sys`. On Windows
MSVC that crate's build script requires NASM and panics with `NASM command not found! Build cannot
continue.` This happens even with `--no-default-features --features kv-rocksdb`, so it cannot be
avoided by feature selection on `surrealdb` itself.

Add a direct dependency on `aws-lc-sys` to the **workspace root** `Cargo.toml` purely to turn on
its pre-generated assembly, which feature unification then applies to the transitive copy:

```toml
[workspace.dependencies]
# Not used directly. `prebuilt-nasm` removes the NASM build requirement that
# surrealdb-core -> jsonwebtoken -> aws-lc-rs imposes on Windows MSVC.
aws-lc-sys = { version = "0.43", features = ["prebuilt-nasm"] }
```

and reference it from `crates/planning-store/Cargo.toml`:

```toml
aws-lc-sys = { workspace = true }
```

The alternative — `winget install NASM.NASM` and putting it on `PATH` — also works and avoids the
unused-dependency smell. Choose it if `fallow` or clippy objects to a dependency that is never
named in code; add a `fallow-ignore` for the crate if you keep the feature-unification approach.

- [ ] **Step 3: Verify the toolchain end to end before writing any code**

```bash
cargo new --lib /tmp/surreal-probe && cd /tmp/surreal-probe
cargo add surrealdb --no-default-features --features kv-rocksdb
cargo add aws-lc-sys --features prebuilt-nasm
cargo check
```

Expected: `Finished`. First build compiles RocksDB from C++ source — budget 10–20 minutes and do
not interrupt it. If it fails, fix the toolchain here rather than inside the real workspace, where
the failure will be tangled up with your own code.

- [ ] **Step 4: Record the finding**

Write `docs/lessons-learned/surrealdb-rocksdb-windows-build-prerequisites.md` covering both
blockers, their exact error text, the two fixes, and the first-build duration. Register it in
`docs/lessons-learned/README.md`. A future contributor hitting `Unable to find libclang` has no way
to connect it to SurrealDB without this note.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml docs/lessons-learned
git commit -m "build: document and fix Windows prerequisites for SurrealDB with RocksDB"
```

---

### Task 1: `planning-core` — identity and clock

**Files:**
- Create: `crates/planning-core/Cargo.toml`, `src/lib.rs`, `src/private/mod.rs`,
  `src/private/ids.rs`, `src/private/clock.rs`, `src/private/system_clock.rs`
- Modify: `Cargo.toml` (workspace members)
- Test: inline `#[cfg(test)]` modules in `ids.rs` and `clock.rs`

**Interfaces:**
- Consumes: the workspace dependency table from plan 0002.
- Produces, all re-exported from `planning_core`:
  - `ValueId, GoalId, HabitId, TaskId, RecurringTaskId, AssociationId, DailyPlanId,
    WeeklyFocusId, WeeklyReviewId, HabitCheckInId` — each with
    `const TABLE: &'static str`, `generate() -> Self`, `new(impl Into<String>) -> Self`,
    `as_str(&self) -> &str`, `Display`, and transparent serde.
  - `trait Clock { fn now(&self) -> DateTime<Utc>; }` (`Send + Sync`)
  - `struct FixedClock` with `at(DateTime<Utc>)`, `set(DateTime<Utc>)`, `advance(Duration)`
  - `struct SystemClock`

- [ ] **Step 1: Add the crate to the workspace**

In root `Cargo.toml`, set `members = ["src-tauri", "crates/planning-core"]` and append to
`[workspace.dependencies]`:

```toml
planning-core = { path = "crates/planning-core" }
```

`crates/planning-core/Cargo.toml`:

```toml
[package]
name = "planning-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
chrono = { workspace = true }
chrono-tz = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
```

- [ ] **Step 2: Write the failing id test**

`crates/planning-core/src/private/ids.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_carry_their_table() {
        assert_eq!(TaskId::TABLE, "task");
        assert_ne!(TaskId::generate(), TaskId::generate());
    }

    #[test]
    fn ids_serialize_as_bare_strings() {
        let id = TaskId::new("abc");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"abc\"");
        assert_eq!(id.to_string(), "abc");
    }
}
```

Add `serde_json = { workspace = true }` under `[dev-dependencies]` in the crate manifest.

- [ ] **Step 3: Run the test to verify it fails**

```bash
cargo test -p planning-core
```

Expected: FAIL — `cannot find type 'TaskId' in this scope`.

- [ ] **Step 4: Implement `ids.rs`**

```rust
use serde::{Deserialize, Serialize};

/// Declares a transparent, table-aware identifier newtype.
///
/// Distinct types per entity make it a compile error to pass a `GoalId` where a
/// `TaskId` belongs — the cheapest guard available against link-table mistakes.
macro_rules! define_ids {
    ($($name:ident => $table:literal),+ $(,)?) => { $(
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub const TABLE: &'static str = $table;

            pub fn new(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }

            /// UUID v7 keeps ids time-ordered, which keeps RocksDB key locality sane.
            pub fn generate() -> Self {
                Self(uuid::Uuid::now_v7().to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    )+ };
}

define_ids! {
    ValueId => "value",
    GoalId => "goal",
    HabitId => "habit",
    TaskId => "task",
    RecurringTaskId => "recurring_task",
    AssociationId => "association",
    DailyPlanId => "daily_plan",
    WeeklyFocusId => "weekly_focus",
    WeeklyReviewId => "weekly_review",
    HabitCheckInId => "habit_check_in",
}
```

- [ ] **Step 5: Write the failing clock test**

`crates/planning-core/src/private/clock.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_holds_and_advances_its_instant() {
        let start = Utc.with_ymd_and_hms(2026, 8, 6, 5, 0, 0).unwrap();
        let clock = FixedClock::at(start);
        assert_eq!(clock.now(), start);

        clock.advance(Duration::hours(3));
        assert_eq!(clock.now(), Utc.with_ymd_and_hms(2026, 8, 6, 8, 0, 0).unwrap());
    }
}
```

- [ ] **Step 6: Run the test to verify it fails**

```bash
cargo test -p planning-core
```

Expected: FAIL — `cannot find struct 'FixedClock'`.

- [ ] **Step 7: Implement `clock.rs` and `system_clock.rs`**

`clock.rs`:

```rust
use chrono::{DateTime, Duration, Utc};
use std::sync::Mutex;

/// The single source of "now". Production code never reads the wall clock
/// directly — see `tests/architecture.test.ts`.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Test double. Public because every crate's tests need it.
pub struct FixedClock {
    instant: Mutex<DateTime<Utc>>,
}

impl FixedClock {
    pub fn at(instant: DateTime<Utc>) -> Self {
        Self { instant: Mutex::new(instant) }
    }

    pub fn set(&self, instant: DateTime<Utc>) {
        *self.instant.lock().expect("clock mutex poisoned") = instant;
    }

    pub fn advance(&self, delta: Duration) {
        let mut guard = self.instant.lock().expect("clock mutex poisoned");
        *guard += delta;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        *self.instant.lock().expect("clock mutex poisoned")
    }
}
```

`system_clock.rs` — the only file permitted to call `Utc::now()`:

```rust
use super::clock::Clock;
use chrono::{DateTime, Utc};

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
```

- [ ] **Step 8: Write `private/mod.rs` and `lib.rs`**

`private/mod.rs`:

```rust
pub mod clock;
pub mod ids;
pub mod system_clock;
```

`lib.rs` — interface only, no logic:

```rust
//! Pure planning vocabulary: identity, time, and (from plan 0004) entities.
//! This crate performs no IO.

mod private;

pub use private::clock::{Clock, FixedClock};
pub use private::ids::{
    AssociationId, DailyPlanId, GoalId, HabitCheckInId, HabitId, RecurringTaskId, TaskId, ValueId,
    WeeklyFocusId, WeeklyReviewId,
};
pub use private::system_clock::SystemClock;
```

- [ ] **Step 9: Run the tests to verify they pass**

```bash
cargo test -p planning-core
```

Expected: PASS — 3 tests.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock crates/planning-core
git commit -m "feat(core): add typed ids and the Clock abstraction"
```

---

### Task 2: `CalendarWeek` and `HomeCalendar`

**Files:**
- Create: `crates/planning-core/src/private/calendar_week.rs`,
  `crates/planning-core/src/private/home_calendar.rs`
- Modify: `crates/planning-core/src/private/mod.rs`, `crates/planning-core/src/lib.rs`

**Interfaces:**
- Consumes: `Clock` from Task 1.
- Produces:
  - `CalendarWeek` — `containing(NaiveDate) -> Self`, `monday() -> NaiveDate`,
    `sunday() -> NaiveDate`, `label() -> String` (`"2026-W32"`), `parse(&str) -> Result<Self, CalendarError>`,
    `next() -> Self`, `previous() -> Self`, `contains(NaiveDate) -> bool`. `Copy`, `Ord`, serde as
    its label string.
  - `HomeCalendar` — `new(Tz)`, `zone() -> Tz`, `today(&dyn Clock) -> NaiveDate`,
    `current_week(&dyn Clock) -> CalendarWeek`, `weekday(NaiveDate) -> Weekday`.
  - `CalendarError` (thiserror) with variant `InvalidWeekLabel(String)`.

Every downstream plan gets its dates from `HomeCalendar::today` and its weeks from
`CalendarWeek::containing`. Nothing computes a week boundary by hand.

- [ ] **Step 1: Write the failing `CalendarWeek` test**

These fixtures were verified against the ISO-8601 week algorithm — including 2026, which has 53
weeks. Do not "simplify" them.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn weeks_run_monday_to_sunday() {
        let week = CalendarWeek::containing(date(2026, 8, 6));
        assert_eq!(week.label(), "2026-W32");
        assert_eq!(week.monday(), date(2026, 8, 3));
        assert_eq!(week.sunday(), date(2026, 8, 9));
        assert!(week.contains(date(2026, 8, 9)));
        assert!(!week.contains(date(2026, 8, 10)));
    }

    #[test]
    fn iso_years_do_not_follow_the_calendar_year() {
        // 29 Dec 2025 is a Monday belonging to ISO week 2026-W01.
        assert_eq!(CalendarWeek::containing(date(2025, 12, 29)).label(), "2026-W01");
        assert_eq!(CalendarWeek::containing(date(2025, 12, 28)).label(), "2025-W52");
        // 3 Jan 2027 is a Sunday still inside 2026-W53 — 2026 is a 53-week year.
        assert_eq!(CalendarWeek::containing(date(2027, 1, 3)).label(), "2026-W53");
    }

    #[test]
    fn next_and_previous_cross_the_year_boundary() {
        let last = CalendarWeek::containing(date(2027, 1, 3));
        assert_eq!(last.label(), "2026-W53");
        assert_eq!(last.next().label(), "2027-W01");
        assert_eq!(CalendarWeek::containing(date(2026, 1, 5)).previous().label(), "2026-W01");
    }

    #[test]
    fn labels_round_trip() {
        let week = CalendarWeek::containing(date(2026, 8, 6));
        assert_eq!(CalendarWeek::parse("2026-W32").unwrap(), week);
        assert!(CalendarWeek::parse("2026-32").is_err());
        assert!(CalendarWeek::parse("2026-W99").is_err());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cargo test -p planning-core calendar
```

Expected: FAIL — `cannot find struct 'CalendarWeek'`.

- [ ] **Step 3: Implement `calendar_week.rs`**

```rust
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CalendarError {
    #[error("'{0}' is not a valid ISO week label such as 2026-W32")]
    InvalidWeekLabel(String),
}

/// A Monday-through-Sunday ISO-8601 week. Serialized as its label so report
/// front matter and database records stay human-readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CalendarWeek {
    iso_year: i32,
    iso_week: u32,
}

impl CalendarWeek {
    pub fn containing(date: NaiveDate) -> Self {
        let iso = date.iso_week();
        Self { iso_year: iso.year(), iso_week: iso.week() }
    }

    pub fn monday(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.iso_year, self.iso_week, Weekday::Mon)
            .expect("CalendarWeek can only hold weeks that exist")
    }

    pub fn sunday(&self) -> NaiveDate {
        self.monday() + Duration::days(6)
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.monday() && date <= self.sunday()
    }

    pub fn label(&self) -> String {
        format!("{}-W{:02}", self.iso_year, self.iso_week)
    }

    pub fn next(&self) -> Self {
        Self::containing(self.monday() + Duration::days(7))
    }

    pub fn previous(&self) -> Self {
        Self::containing(self.monday() - Duration::days(7))
    }

    pub fn parse(label: &str) -> Result<Self, CalendarError> {
        let invalid = || CalendarError::InvalidWeekLabel(label.to_string());
        let (year, week) = label.split_once("-W").ok_or_else(invalid)?;
        let iso_year: i32 = year.parse().map_err(|_| invalid())?;
        let iso_week: u32 = week.parse().map_err(|_| invalid())?;
        // Rejects W00, W54, and W53 in 52-week years in one shot.
        NaiveDate::from_isoywd_opt(iso_year, iso_week, Weekday::Mon).ok_or_else(invalid)?;
        Ok(Self { iso_year, iso_week })
    }
}

impl TryFrom<String> for CalendarWeek {
    type Error = CalendarError;

    fn try_from(label: String) -> Result<Self, Self::Error> {
        Self::parse(&label)
    }
}

impl From<CalendarWeek> for String {
    fn from(week: CalendarWeek) -> Self {
        week.label()
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
cargo test -p planning-core calendar
```

Expected: PASS — 4 tests.

- [ ] **Step 5: Write the failing `HomeCalendar` test**

`crates/planning-core/src/private/home_calendar.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;
    use chrono_tz::Tz;

    /// 2026-08-07 01:30 UTC. In Madrid (UTC+2 in August) that is already the 7th
    /// at 03:30; in Los Angeles (UTC-7) it is still the 6th at 18:30.
    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 1, 30, 0).unwrap())
    }

    #[test]
    fn today_follows_the_home_zone_not_utc() {
        let madrid = HomeCalendar::new(Tz::Europe__Madrid);
        assert_eq!(madrid.today(&clock()), NaiveDate::from_ymd_opt(2026, 8, 7).unwrap());

        let los_angeles = HomeCalendar::new(Tz::America__Los_Angeles);
        assert_eq!(los_angeles.today(&clock()), NaiveDate::from_ymd_opt(2026, 8, 6).unwrap());
    }

    #[test]
    fn the_home_zone_can_move_the_calendar_week() {
        // 2026-08-10 00:30 UTC is Monday of W33 in Madrid but still Sunday of W32 in Los Angeles.
        let instant = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 10, 0, 30, 0).unwrap());
        assert_eq!(HomeCalendar::new(Tz::Europe__Madrid).current_week(&instant).label(), "2026-W33");
        assert_eq!(
            HomeCalendar::new(Tz::America__Los_Angeles).current_week(&instant).label(),
            "2026-W32"
        );
    }
}
```

- [ ] **Step 6: Run the test to verify it fails**

```bash
cargo test -p planning-core home_calendar
```

Expected: FAIL — `cannot find struct 'HomeCalendar'`.

- [ ] **Step 7: Implement `home_calendar.rs`**

```rust
use super::calendar_week::CalendarWeek;
use super::clock::Clock;
use chrono::{Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;

/// Projects instants onto dates in the synchronized home time zone.
///
/// Every date, week, deadline, and recurrence decision in the app goes through
/// here. Device time zones are never consulted (ADR 0001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeCalendar {
    zone: Tz,
}

impl HomeCalendar {
    pub fn new(zone: Tz) -> Self {
        Self { zone }
    }

    pub fn zone(&self) -> Tz {
        self.zone
    }

    pub fn today(&self, clock: &dyn Clock) -> NaiveDate {
        clock.now().with_timezone(&self.zone).date_naive()
    }

    pub fn current_week(&self, clock: &dyn Clock) -> CalendarWeek {
        CalendarWeek::containing(self.today(clock))
    }

    pub fn weekday(&self, date: NaiveDate) -> Weekday {
        date.weekday()
    }
}
```

Remove the unused `Utc` import if clippy flags it.

- [ ] **Step 8: Export from `lib.rs` and run the full crate**

Add to `private/mod.rs`: `pub mod calendar_week; pub mod home_calendar;`
Add to `lib.rs`:

```rust
pub use private::calendar_week::{CalendarError, CalendarWeek};
pub use private::home_calendar::HomeCalendar;
```

```bash
cargo test -p planning-core && cargo clippy -p planning-core --all-targets -- -D warnings
```

Expected: PASS — 9 tests, no warnings.

- [ ] **Step 9: Commit**

```bash
git add crates/planning-core
git commit -m "feat(core): add CalendarWeek and home-zone-governed HomeCalendar"
```

---

### Task 3: Device settings

**Files:**
- Create: `crates/planning-store/Cargo.toml`, `src/lib.rs`, `src/private/mod.rs`,
  `src/private/error.rs`, `src/private/device_settings.rs`
- Modify: root `Cargo.toml`

**Interfaces:**
- Consumes: nothing from planning-core yet.
- Produces:
  - `DeviceSettings { device_id: String, device_name: String, sync_folder: Option<PathBuf>,
    launch_time: NaiveTime, retry_window_minutes: u32, last_missed_prompt: Option<NaiveDate> }`
    with `Default` (07:00, 240 minutes, generated id).
  - `DeviceSettingsFile` — `at(PathBuf) -> Self`, `load(&self) -> Result<DeviceSettings, StoreError>`
    (creates defaults on first run), `save(&self, &DeviceSettings) -> Result<(), StoreError>`,
    and `default_path() -> Result<PathBuf, StoreError>`.
  - `StoreError` (thiserror): `Io(#[from] std::io::Error)`, `Corrupt { path: PathBuf, detail: String }`,
    `NoConfigDirectory`, `NotReady(StoreHealth)` (added in Task 6), `Database(String)`.

Plan 0008 reads and writes `last_missed_prompt` through this same type — the launcher shares the
settings file with the app.

- [ ] **Step 1: Create the crate and add it to the workspace**

Set `members = ["src-tauri", "crates/planning-core", "crates/planning-store"]` and add
`planning-store = { path = "crates/planning-store" }` to `[workspace.dependencies]`.

`crates/planning-store/Cargo.toml`:

```toml
[package]
name = "planning-store"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
planning-core = { workspace = true }
chrono = { workspace = true }
chrono-tz = { workspace = true }
directories = "6.0.0"
serde = { workspace = true }
serde_json = { workspace = true }
surrealdb = { version = "3.2.4", default-features = false, features = ["kv-rocksdb"] }
thiserror = { workspace = true }
uuid = { workspace = true }

[dev-dependencies]
tempfile = "3.27.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 2: Write the failing test**

`crates/planning-store/src/private/device_settings.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn first_load_creates_defaults_and_persists_a_stable_device_id() {
        let dir = TempDir::new().unwrap();
        let file = DeviceSettingsFile::at(dir.path().join("device-settings.json"));

        let first = file.load().unwrap();
        assert_eq!(first.launch_time, NaiveTime::from_hms_opt(7, 0, 0).unwrap());
        assert_eq!(first.retry_window_minutes, 240);
        assert_eq!(first.sync_folder, None);

        let second = file.load().unwrap();
        assert_eq!(second.device_id, first.device_id, "device id must survive reload");
    }

    #[test]
    fn saved_settings_round_trip() {
        let dir = TempDir::new().unwrap();
        let file = DeviceSettingsFile::at(dir.path().join("device-settings.json"));

        let mut settings = file.load().unwrap();
        settings.sync_folder = Some(PathBuf::from("/drive/self-planning"));
        settings.launch_time = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        file.save(&settings).unwrap();

        let reloaded = file.load().unwrap();
        assert_eq!(reloaded.sync_folder, settings.sync_folder);
        assert_eq!(reloaded.launch_time, settings.launch_time);
    }

    #[test]
    fn a_corrupt_file_reports_its_path_instead_of_silently_resetting() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("device-settings.json");
        std::fs::write(&path, "{ not json").unwrap();

        let error = DeviceSettingsFile::at(path.clone()).load().unwrap_err();
        assert!(matches!(error, StoreError::Corrupt { path: p, .. } if p == path));
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

```bash
cargo test -p planning-store device_settings
```

Expected: FAIL — `cannot find struct 'DeviceSettingsFile'`.

- [ ] **Step 4: Implement `error.rs`**

```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("could not read or write {0}")]
    Io(#[from] std::io::Error),

    #[error("{path} is not readable as settings: {detail}")]
    Corrupt { path: PathBuf, detail: String },

    #[error("this operating system reported no configuration directory")]
    NoConfigDirectory,

    #[error("database error: {0}")]
    Database(String),
}

impl From<surrealdb::Error> for StoreError {
    fn from(error: surrealdb::Error) -> Self {
        StoreError::Database(error.to_string())
    }
}
```

- [ ] **Step 5: Implement `device_settings.rs`**

```rust
use super::error::StoreError;
use chrono::{NaiveDate, NaiveTime};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Per-device configuration. Deliberately stored OUTSIDE the Synchronization
/// Folder: launch time, retry window, and folder path are device facts and must
/// never travel between machines (ADR 0001).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub device_id: String,
    pub device_name: String,
    pub sync_folder: Option<PathBuf>,
    pub launch_time: NaiveTime,
    pub retry_window_minutes: u32,
    pub last_missed_prompt: Option<NaiveDate>,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            device_id: uuid::Uuid::now_v7().to_string(),
            device_name: hostname(),
            sync_folder: None,
            launch_time: NaiveTime::from_hms_opt(7, 0, 0).expect("07:00 is a valid time"),
            retry_window_minutes: 240,
            last_missed_prompt: None,
        }
    }
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-device".to_string())
}

pub struct DeviceSettingsFile {
    path: PathBuf,
}

impl DeviceSettingsFile {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The OS config directory — never the Synchronization Folder.
    pub fn default_path() -> Result<PathBuf, StoreError> {
        let dirs = ProjectDirs::from("com", "gamecoderstudios", "self-planning")
            .ok_or(StoreError::NoConfigDirectory)?;
        Ok(dirs.config_dir().join("device-settings.json"))
    }

    /// Reads settings, creating defaults on first run so callers never handle "missing".
    pub fn load(&self) -> Result<DeviceSettings, StoreError> {
        if !self.path.exists() {
            let settings = DeviceSettings::default();
            self.save(&settings)?;
            return Ok(settings);
        }
        let text = std::fs::read_to_string(&self.path)?;
        serde_json::from_str(&text).map_err(|error| StoreError::Corrupt {
            path: self.path.clone(),
            detail: error.to_string(),
        })
    }

    pub fn save(&self, settings: &DeviceSettings) -> Result<(), StoreError> {
        create_parent(&self.path)?;
        let text = serde_json::to_string_pretty(settings).map_err(|error| StoreError::Corrupt {
            path: self.path.clone(),
            detail: error.to_string(),
        })?;
        std::fs::write(&self.path, text)?;
        Ok(())
    }
}

fn create_parent(path: &Path) -> Result<(), StoreError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    Ok(())
}
```

- [ ] **Step 6: Run the tests to verify they pass**

```bash
cargo test -p planning-store device_settings
```

Expected: PASS — 3 tests.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/planning-store
git commit -m "feat(store): add device-local settings kept outside the sync folder"
```

---

### Task 4: The SurrealDB connection

**Files:**
- Create: `crates/planning-store/src/private/database.rs`
- Modify: `crates/planning-store/src/private/mod.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: `StoreError` from Task 3.
- Produces:
  - `Database` — `open(&Path) -> Result<Database, StoreError>` (async), `inner(&self) -> &Surreal<Db>`.
  - `Database::DIRECTORY: &str = "planning-db"` — the sub-directory of the Synchronization Folder
    holding RocksDB.
  - Namespace `planning`, database `planning`.

Plans 0004–0006 take `&Database` and run their own queries. `Database` never gains
domain-specific methods.

- [ ] **Step 1: Verify the SurrealDB API before writing against it**

The embedded RocksDB entry point changed shape between SurrealDB major versions. Confirm it
compiles before building on it:

```bash
cargo check -p planning-store
```

The expected 3.x API is `Surreal::new::<RocksDb>(path)` from `surrealdb::engine::local::{Db, RocksDb}`.
If that path does not resolve, run `cargo doc -p surrealdb --open` and search for `engine::local`
to find the current name, then adjust the code below and note the difference in the commit
message. Do not downgrade the crate.

First compilation pulls and builds RocksDB from C++ source and can take several minutes. This is
expected once, not per build.

- [ ] **Step 2: Write the failing test**

`crates/planning-store/src/private/database.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Probe {
        note: String,
    }

    #[tokio::test]
    async fn a_record_written_by_one_instance_is_read_by_the_next() {
        let folder = TempDir::new().unwrap();

        let first = Database::open(folder.path()).await.unwrap();
        let _: Option<Probe> = first
            .inner()
            .create(("probe", "one"))
            .content(Probe { note: "hello".into() })
            .await
            .unwrap();
        drop(first);

        // Reopening the same directory is exactly what a second device does
        // after Google Drive has synchronized it.
        let second = Database::open(folder.path()).await.unwrap();
        let found: Option<Probe> = second.inner().select(("probe", "one")).await.unwrap();
        assert_eq!(found, Some(Probe { note: "hello".into() }));
    }

    #[tokio::test]
    async fn opening_creates_the_database_directory_under_the_sync_folder() {
        let folder = TempDir::new().unwrap();
        let _database = Database::open(folder.path()).await.unwrap();
        assert!(folder.path().join(Database::DIRECTORY).is_dir());
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

```bash
cargo test -p planning-store database
```

Expected: FAIL — `cannot find struct 'Database'`.

- [ ] **Step 4: Implement `database.rs`**

```rust
use super::error::StoreError;
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

/// The embedded SurrealDB connection. Owns no domain knowledge — callers bring
/// their own queries (ADR 0001).
pub struct Database {
    inner: Surreal<Db>,
}

impl Database {
    /// Sub-directory of the Synchronization Folder holding the RocksDB files.
    /// Keeping it in a named sub-directory leaves room for `weekly-reports/`
    /// and the writer lock beside it.
    pub const DIRECTORY: &'static str = "planning-db";

    const NAMESPACE: &'static str = "planning";
    const DATABASE: &'static str = "planning";

    pub async fn open(sync_folder: &Path) -> Result<Self, StoreError> {
        let path = sync_folder.join(Self::DIRECTORY);
        std::fs::create_dir_all(&path)?;
        let inner = Surreal::new::<RocksDb>(path).await?;
        inner.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Self { inner })
    }

    pub fn inner(&self) -> &Surreal<Db> {
        &self.inner
    }
}
```

If `Surreal::new::<RocksDb>` rejects a `PathBuf`, pass `path.to_string_lossy().as_ref()`.

- [ ] **Step 5: Run the tests to verify they pass**

```bash
cargo test -p planning-store database
```

Expected: PASS — 2 tests. **This proves acceptance criterion A1** at the storage layer: data
written by one instance and then closed is readable by the next instance opening the same
directory. Plan 0005 re-proves it end-to-end with a real Task.

- [ ] **Step 6: Commit**

```bash
git add crates/planning-store Cargo.lock
git commit -m "feat(store): open embedded SurrealDB on RocksDB in the sync folder"
```

---

### Task 5: The synchronized home time zone

**Files:**
- Create: `crates/planning-store/src/private/home_settings.rs`
- Modify: `crates/planning-store/src/private/mod.rs`, `src/lib.rs`

**Interfaces:**
- Consumes: `Database` from Task 4, `Clock` from Task 1.
- Produces:
  - `HomeSettings { home_zone: Option<Tz>, updated_at: DateTime<Utc> }`
  - `HomeSettingsRepository::load(&Database) -> Result<HomeSettings, StoreError>` (async) —
    returns `home_zone: None` when never set.
  - `HomeSettingsRepository::set_zone(&Database, SetZone) -> Result<HomeSettings, StoreError>`
    where `SetZone { zone: Tz, clock: &dyn Clock }` — a struct because the 3-parameter limit
    applies and `&Database` already counts.

The home zone is **synchronized** (it lives in the database, not the device file) so every device
computes identical dates and weeks. `home_zone: None` is what makes `StoreHealth::SetupIncomplete`
in Task 6.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use planning_core::FixedClock;
    use tempfile::TempDir;

    async fn database() -> (TempDir, Database) {
        let folder = TempDir::new().unwrap();
        let database = Database::open(folder.path()).await.unwrap();
        (folder, database)
    }

    #[tokio::test]
    async fn the_home_zone_starts_unset() {
        let (_folder, database) = database().await;
        let settings = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(settings.home_zone, None);
    }

    #[tokio::test]
    async fn setting_the_zone_persists_it_with_its_timestamp() {
        let (_folder, database) = database().await;
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());

        let saved = HomeSettingsRepository::set_zone(
            &database,
            SetZone { zone: Tz::Europe__Madrid, clock: &clock },
        )
        .await
        .unwrap();
        assert_eq!(saved.home_zone, Some(Tz::Europe__Madrid));
        assert_eq!(saved.updated_at, clock.now());

        let reloaded = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(reloaded.home_zone, Some(Tz::Europe__Madrid));
    }

    #[tokio::test]
    async fn changing_the_zone_replaces_it_rather_than_appending() {
        let (_folder, database) = database().await;
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());
        for zone in [Tz::Europe__Madrid, Tz::America__Los_Angeles] {
            HomeSettingsRepository::set_zone(&database, SetZone { zone, clock: &clock })
                .await
                .unwrap();
        }
        let reloaded = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(reloaded.home_zone, Some(Tz::America__Los_Angeles));
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cargo test -p planning-store home_settings
```

Expected: FAIL — `cannot find struct 'HomeSettingsRepository'`.

- [ ] **Step 3: Implement `home_settings.rs`**

```rust
use super::database::Database;
use super::error::StoreError;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use planning_core::Clock;
use serde::{Deserialize, Serialize};

/// Settings that must be identical on every device. Stored in the synchronized
/// database, not the device settings file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeSettings {
    pub home_zone: Option<Tz>,
    pub updated_at: DateTime<Utc>,
}

pub struct SetZone<'a> {
    pub zone: Tz,
    pub clock: &'a dyn Clock,
}

pub struct HomeSettingsRepository;

impl HomeSettingsRepository {
    const TABLE: &'static str = "settings";
    const RECORD: &'static str = "home";

    pub async fn load(database: &Database) -> Result<HomeSettings, StoreError> {
        let found: Option<HomeSettings> =
            database.inner().select((Self::TABLE, Self::RECORD)).await?;
        Ok(found.unwrap_or(HomeSettings {
            home_zone: None,
            updated_at: DateTime::<Utc>::MIN_UTC,
        }))
    }

    /// Upserts the single settings record. There is exactly one, by construction,
    /// so a second device writing a zone replaces rather than duplicates it.
    pub async fn set_zone(
        database: &Database,
        request: SetZone<'_>,
    ) -> Result<HomeSettings, StoreError> {
        let settings = HomeSettings {
            home_zone: Some(request.zone),
            updated_at: request.clock.now(),
        };
        let saved: Option<HomeSettings> = database
            .inner()
            .upsert((Self::TABLE, Self::RECORD))
            .content(settings.clone())
            .await?;
        Ok(saved.unwrap_or(settings))
    }
}
```

If `upsert` is unavailable in the installed SurrealDB version, use
`database.inner().query("UPSERT settings:home CONTENT $data").bind(("data", settings.clone()))`
and take row 0. Do **not** substitute delete-then-create — ADR 0002 forbids `DELETE`.

- [ ] **Step 4: Run the tests to verify they pass**

```bash
cargo test -p planning-store home_settings
```

Expected: PASS — 3 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/planning-store
git commit -m "feat(store): synchronize the home time zone across devices"
```

---

### Task 6: Sync safety — writer lock, conflict detection, `StoreHealth`

**Files:**
- Create: `crates/planning-store/src/private/writer_lock.rs`,
  `crates/planning-store/src/private/conflicts.rs`, `crates/planning-store/src/private/health.rs`
- Modify: `crates/planning-store/src/private/mod.rs`, `src/lib.rs`, `src/private/error.rs`

**Interfaces:**
- Consumes: `Database`, `HomeSettingsRepository`, `DeviceSettings`, `Clock`.
- Produces:
  - ```rust
    pub enum StoreHealth {
        Ready,
        SetupIncomplete { reason: SetupGap },
        FolderMissing { path: PathBuf },
        LockedByAnotherDevice { device_name: String, since: DateTime<Utc> },
        SyncConflict { artifacts: Vec<PathBuf> },
        Unreadable { detail: String },
    }
    pub enum SetupGap { NoSyncFolder, NoHomeZone }
    ```
  - `WriterLock` — `acquire(&Path, AcquireLock) -> Result<WriterLock, StoreHealth>`,
    `heartbeat(&self, &dyn Clock)`, `release(self)`. `Drop` releases.
    `AcquireLock { settings: &DeviceSettings, clock: &dyn Clock }`.
  - `WriterLock::STALE_AFTER_MINUTES: i64 = 15`
  - `conflicts::scan(&Path) -> Vec<PathBuf>`
  - `StoreError::NotReady(StoreHealth)`

`StoreHealth::Ready` is the only value that permits a write. Plan 0004's `PlanningApp` holds the
lock for its lifetime; plan 0008's launcher never acquires one because it only reads.

- [ ] **Step 1: Write the failing conflict-detection test**

`crates/planning-store/src/private/conflicts.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn google_drive_conflict_artifacts_are_detected() {
        let folder = TempDir::new().unwrap();
        for name in [
            "planning-db",
            "CURRENT (1)",
            "MANIFEST-000004 (conflicted copy 2026-08-06)",
            "writer.lock",
        ] {
            std::fs::write(folder.path().join(name), "").unwrap();
        }

        let found: Vec<String> = scan(folder.path())
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_string))
            .collect();

        assert!(found.contains(&"CURRENT (1)".to_string()));
        assert!(found.contains(&"MANIFEST-000004 (conflicted copy 2026-08-06)".to_string()));
        assert_eq!(found.len(), 2, "clean files must not be reported: {found:?}");
    }

    #[test]
    fn a_clean_folder_reports_nothing() {
        let folder = TempDir::new().unwrap();
        std::fs::create_dir(folder.path().join("planning-db")).unwrap();
        assert!(scan(folder.path()).is_empty());
    }
}
```

- [ ] **Step 2: Run it to verify it fails, then implement `conflicts.rs`**

```bash
cargo test -p planning-store conflicts
```

Expected: FAIL — `cannot find function 'scan'`.

```rust
use std::path::{Path, PathBuf};

/// Google Drive for desktop renames a conflicting file rather than merging it.
/// Either pattern inside the Synchronization Folder means the database may be
/// torn, so the app must refuse to write (ADR 0001).
fn is_conflict_artifact(name: &str) -> bool {
    if name.contains("(conflicted copy") {
        return true;
    }
    // " (1)", " (2)" ... appended before or instead of an extension.
    let Some(open) = name.rfind(" (") else {
        return false;
    };
    let rest = &name[open + 2..];
    let Some(close) = rest.find(')') else {
        return false;
    };
    !rest[..close].is_empty() && rest[..close].chars().all(|c| c.is_ascii_digit())
}

/// Walks the Synchronization Folder one level deep plus the database directory.
pub fn scan(sync_folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in [sync_folder.to_path_buf(), sync_folder.join("planning-db")] {
        collect_from(&root, &mut found);
    }
    found
}

fn collect_from(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_conflict_artifact(name) {
            continue;
        }
        found.push(path);
    }
}
```

Re-run: expected PASS — 2 tests.

- [ ] **Step 3: Write the failing writer-lock test**

`crates/planning-store/src/private/writer_lock.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use planning_core::FixedClock;
    use tempfile::TempDir;

    fn settings(name: &str) -> DeviceSettings {
        DeviceSettings { device_name: name.to_string(), ..DeviceSettings::default() }
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    #[test]
    fn a_second_device_is_refused_while_the_lock_is_fresh() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let desktop = settings("desktop");
        let clock = clock();

        let _held = WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &laptop, clock: &clock },
        )
        .expect("first acquire succeeds");

        let refused = WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &desktop, clock: &clock },
        )
        .unwrap_err();

        assert!(matches!(
            refused,
            StoreHealth::LockedByAnotherDevice { ref device_name, .. } if device_name == "laptop"
        ));
    }

    #[test]
    fn a_stale_lock_can_be_taken_over() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let desktop = settings("desktop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &laptop, clock: &clock },
        )
        .unwrap();
        std::mem::forget(held); // simulate a crash: the lock file is left behind

        clock.advance(Duration::minutes(WriterLock::STALE_AFTER_MINUTES + 1));
        assert!(WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &desktop, clock: &clock }
        )
        .is_ok());
    }

    #[test]
    fn the_same_device_reacquires_its_own_lock() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &laptop, clock: &clock },
        )
        .unwrap();
        std::mem::forget(held);

        assert!(WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &laptop, clock: &clock }
        )
        .is_ok());
    }

    #[test]
    fn releasing_removes_the_lock_file() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock { settings: &laptop, clock: &clock },
        )
        .unwrap();
        assert!(folder.path().join(WriterLock::FILE_NAME).exists());
        held.release();
        assert!(!folder.path().join(WriterLock::FILE_NAME).exists());
    }
}
```

- [ ] **Step 4: Run it to verify it fails, then implement `writer_lock.rs`**

```bash
cargo test -p planning-store writer_lock
```

Expected: FAIL — `cannot find struct 'WriterLock'`.

```rust
use super::device_settings::DeviceSettings;
use super::health::StoreHealth;
use chrono::{DateTime, Duration, Utc};
use planning_core::Clock;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LockRecord {
    device_id: String,
    device_name: String,
    heartbeat_at: DateTime<Utc>,
}

pub struct AcquireLock<'a> {
    pub settings: &'a DeviceSettings,
    pub clock: &'a dyn Clock,
}

/// Advisory one-active-writer lock. Google Drive gives no real file locking, so
/// this is a cooperative marker with a heartbeat: a device that crashed leaves a
/// stale record that another device may take over after STALE_AFTER_MINUTES.
pub struct WriterLock {
    path: PathBuf,
    device_id: String,
}

impl WriterLock {
    pub const FILE_NAME: &'static str = "writer.lock";
    pub const STALE_AFTER_MINUTES: i64 = 15;

    pub fn acquire(sync_folder: &Path, request: AcquireLock<'_>) -> Result<Self, StoreHealth> {
        let path = sync_folder.join(Self::FILE_NAME);
        if let Some(holder) = read_lock(&path) {
            let ours = holder.device_id == request.settings.device_id;
            let age = request.clock.now() - holder.heartbeat_at;
            if !ours && age < Duration::minutes(Self::STALE_AFTER_MINUTES) {
                return Err(StoreHealth::LockedByAnotherDevice {
                    device_name: holder.device_name,
                    since: holder.heartbeat_at,
                });
            }
        }
        let lock = Self { path, device_id: request.settings.device_id.clone() };
        lock.write(request.settings, request.clock)?;
        Ok(lock)
    }

    /// Call periodically while the app is open so another device can tell the
    /// difference between "in use" and "crashed".
    pub fn heartbeat(&self, settings: &DeviceSettings, clock: &dyn Clock) {
        let _ = self.write(settings, clock);
    }

    pub fn release(self) {
        drop(self);
    }

    fn write(&self, settings: &DeviceSettings, clock: &dyn Clock) -> Result<(), StoreHealth> {
        let record = LockRecord {
            device_id: self.device_id.clone(),
            device_name: settings.device_name.clone(),
            heartbeat_at: clock.now(),
        };
        let text = serde_json::to_string(&record)
            .map_err(|error| StoreHealth::Unreadable { detail: error.to_string() })?;
        std::fs::write(&self.path, text)
            .map_err(|error| StoreHealth::Unreadable { detail: error.to_string() })
    }
}

impl Drop for WriterLock {
    fn drop(&mut self) {
        // Best effort: a lost lock file only means the next device waits out the
        // staleness window. Never panic on shutdown.
        let _ = std::fs::remove_file(&self.path);
    }
}

fn read_lock(path: &Path) -> Option<LockRecord> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}
```

`remove_file` on a lock file is deletion of a transient marker, not of planning data — ADR 0002's
no-hard-delete rule covers entities, not lock files. Note this in the architecture doc.

Re-run: expected PASS — 4 tests.

- [ ] **Step 5: Write the failing health test and implement `health.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn setup_is_incomplete_without_a_sync_folder() {
        let health = StoreHealth::assess(Assessment {
            sync_folder: None,
            home_zone_is_set: false,
        });
        assert!(matches!(
            health,
            StoreHealth::SetupIncomplete { reason: SetupGap::NoSyncFolder }
        ));
    }

    #[test]
    fn a_missing_folder_is_reported_before_anything_else() {
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(PathBuf::from("/definitely/not/here")),
            home_zone_is_set: true,
        });
        assert!(matches!(health, StoreHealth::FolderMissing { .. }));
    }

    #[test]
    fn a_present_folder_without_a_home_zone_is_incomplete() {
        let folder = TempDir::new().unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: false,
        });
        assert!(matches!(
            health,
            StoreHealth::SetupIncomplete { reason: SetupGap::NoHomeZone }
        ));
    }

    #[test]
    fn conflict_artifacts_block_readiness() {
        let folder = TempDir::new().unwrap();
        std::fs::write(folder.path().join("CURRENT (1)"), "").unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: true,
        });
        assert!(matches!(health, StoreHealth::SyncConflict { .. }));
    }

    #[test]
    fn a_clean_configured_folder_is_ready() {
        let folder = TempDir::new().unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: true,
        });
        assert_eq!(health, StoreHealth::Ready);
        assert!(health.permits_writes());
    }
}
```

Implementation:

```rust
use super::conflicts;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SetupGap {
    NoSyncFolder,
    NoHomeZone,
}

/// Whether the synchronized data can be trusted right now. Only `Ready` permits
/// a write; every other value is a state the UI must show and the launcher must
/// treat as "do not open the app" (ADR 0001).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum StoreHealth {
    Ready,
    SetupIncomplete { reason: SetupGap },
    FolderMissing { path: PathBuf },
    LockedByAnotherDevice { device_name: String, since: DateTime<Utc> },
    SyncConflict { artifacts: Vec<PathBuf> },
    Unreadable { detail: String },
}

pub struct Assessment {
    pub sync_folder: Option<PathBuf>,
    pub home_zone_is_set: bool,
}

impl StoreHealth {
    /// Ordered most-blocking first so the UI always shows the fault the user can
    /// actually act on.
    pub fn assess(assessment: Assessment) -> Self {
        let Some(folder) = assessment.sync_folder else {
            return Self::SetupIncomplete { reason: SetupGap::NoSyncFolder };
        };
        if !folder.is_dir() {
            return Self::FolderMissing { path: folder };
        }
        let artifacts = conflicts::scan(&folder);
        if !artifacts.is_empty() {
            return Self::SyncConflict { artifacts };
        }
        if !assessment.home_zone_is_set {
            return Self::SetupIncomplete { reason: SetupGap::NoHomeZone };
        }
        Self::Ready
    }

    pub fn permits_writes(&self) -> bool {
        matches!(self, Self::Ready)
    }
}
```

Add to `StoreError`:

```rust
    #[error("the synchronized data is not ready for writing")]
    NotReady(StoreHealth),
```

Re-run: expected PASS — 5 tests.

- [ ] **Step 6: Export everything from `lib.rs`**

```rust
//! Persistence for the planning domain: the embedded database, device settings,
//! and the sync-safety gate. Nothing here knows what a Task is.

mod private;

pub use private::database::Database;
pub use private::device_settings::{DeviceSettings, DeviceSettingsFile};
pub use private::error::StoreError;
pub use private::health::{Assessment, SetupGap, StoreHealth};
pub use private::home_settings::{HomeSettings, HomeSettingsRepository, SetZone};
pub use private::writer_lock::{AcquireLock, WriterLock};
```

- [ ] **Step 7: Run the whole crate and commit**

```bash
cargo test -p planning-store && cargo clippy -p planning-store --all-targets -- -D warnings
```

Expected: PASS — 19 tests.

```bash
git add crates/planning-store
git commit -m "feat(store): add writer lock, conflict detection, and the StoreHealth gate"
```

---

### Task 7: `planning-app` — the narrow application API

**Files:**
- Create: `crates/planning-app/Cargo.toml`, `src/lib.rs`, `src/private/mod.rs`,
  `src/private/service.rs`, `src/private/setup.rs`, `src/private/error.rs`
- Modify: root `Cargo.toml`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`,
  `src-tauri/src/private/commands.rs`, `src/lib/api/index.ts`
- Test: `crates/planning-app/src/private/setup.rs` (inline), `src/lib/api/index.test.ts`

**Interfaces:**
- Consumes: everything from `planning-store` and `planning-core`.
- Produces — this is the **only** surface `src-tauri` and `launcher` may call:
  - `PlanningApp::start(StartRequest) -> Result<PlanningApp, AppError>` (async) where
    `StartRequest { settings_path: PathBuf, clock: Arc<dyn Clock> }`
  - `PlanningApp::health(&self) -> StoreHealth`
  - `PlanningApp::choose_sync_folder(&mut self, PathBuf) -> Result<StoreHealth, AppError>` (async)
  - `PlanningApp::set_home_zone(&mut self, Tz) -> Result<StoreHealth, AppError>` (async)
  - `PlanningApp::calendar(&self) -> Result<HomeCalendar, AppError>` — errors unless `Ready`
  - `PlanningApp::database(&self) -> Result<&Database, AppError>` — **crate-visible to plans
    0004–0006 only**; declared `pub(crate)` plus a `#[doc(hidden)]` re-export used by the
    repositories those plans add inside this crate.
  - `AppError` (thiserror): `Store(#[from] StoreError)`, `NotReady(StoreHealth)`, `NoDatabase`.

Plans 0004–0006 add their use cases as methods on `PlanningApp` in new `private/` files. They do
not add new public crates for the binaries to depend on.

- [ ] **Step 1: Create the crate**

Add `crates/planning-app` to workspace members and `planning-app = { path = "crates/planning-app" }`
to `[workspace.dependencies]`.

```toml
[package]
name = "planning-app"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
planning-core = { workspace = true }
planning-store = { workspace = true }
chrono = { workspace = true }
chrono-tz = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3.27.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 2: Write the failing setup test**

`crates/planning-app/src/private/setup.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use planning_core::FixedClock;
    use planning_store::SetupGap;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app(home: &TempDir) -> PlanningApp {
        PlanningApp::start(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(
                Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
            )),
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn a_fresh_install_reports_no_sync_folder() {
        let home = TempDir::new().unwrap();
        let app = app(&home).await;
        assert!(matches!(
            app.health(),
            StoreHealth::SetupIncomplete { reason: SetupGap::NoSyncFolder }
        ));
        assert!(app.calendar().is_err(), "no calendar before setup completes");
    }

    #[tokio::test]
    async fn setup_completes_once_a_folder_and_a_zone_are_chosen() {
        let home = TempDir::new().unwrap();
        let drive = TempDir::new().unwrap();
        let mut app = app(&home).await;

        let after_folder = app.choose_sync_folder(drive.path().to_path_buf()).await.unwrap();
        assert!(matches!(
            after_folder,
            StoreHealth::SetupIncomplete { reason: SetupGap::NoHomeZone }
        ));

        let after_zone = app.set_home_zone(Tz::Europe__Madrid).await.unwrap();
        assert_eq!(after_zone, StoreHealth::Ready);
        assert_eq!(app.calendar().unwrap().zone(), Tz::Europe__Madrid);
    }

    #[tokio::test]
    async fn the_chosen_folder_survives_a_restart() {
        let home = TempDir::new().unwrap();
        let drive = TempDir::new().unwrap();
        {
            let mut app = app(&home).await;
            app.choose_sync_folder(drive.path().to_path_buf()).await.unwrap();
            app.set_home_zone(Tz::Europe__Madrid).await.unwrap();
        }
        let restarted = app(&home).await;
        assert_eq!(restarted.health(), StoreHealth::Ready);
        assert_eq!(restarted.calendar().unwrap().zone(), Tz::Europe__Madrid);
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

```bash
cargo test -p planning-app
```

Expected: FAIL — `cannot find struct 'PlanningApp'`.

- [ ] **Step 4: Implement `error.rs`, `service.rs`, and `setup.rs`**

`error.rs`:

```rust
use planning_store::{StoreError, StoreHealth};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Store(#[from] StoreError),

    #[error("the synchronized data is not ready: {0:?}")]
    NotReady(StoreHealth),

    #[error("no synchronization folder has been chosen yet")]
    NoDatabase,
}
```

`service.rs` — holds the state; keep it under 200 lines by leaving use cases to sibling files:

```rust
use super::error::AppError;
use chrono_tz::Tz;
use planning_core::{Clock, HomeCalendar};
use planning_store::{
    AcquireLock, Assessment, Database, DeviceSettings, DeviceSettingsFile, StoreHealth, WriterLock,
};
use std::path::PathBuf;
use std::sync::Arc;

pub struct StartRequest {
    pub settings_path: PathBuf,
    pub clock: Arc<dyn Clock>,
}

/// The application API. `src-tauri` and `launcher` depend on this and nothing else.
pub struct PlanningApp {
    pub(crate) settings_file: DeviceSettingsFile,
    pub(crate) settings: DeviceSettings,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) database: Option<Database>,
    pub(crate) home_zone: Option<Tz>,
    pub(crate) health: StoreHealth,
    pub(crate) lock: Option<WriterLock>,
}

impl PlanningApp {
    pub fn health(&self) -> StoreHealth {
        self.health.clone()
    }

    /// Available only once setup is complete — this is what makes it impossible
    /// to compute a date before the home zone is known.
    pub fn calendar(&self) -> Result<HomeCalendar, AppError> {
        let zone = self.home_zone.ok_or(AppError::NotReady(self.health.clone()))?;
        if !self.health.permits_writes() {
            return Err(AppError::NotReady(self.health.clone()));
        }
        Ok(HomeCalendar::new(zone))
    }

    pub(crate) fn require_database(&self) -> Result<&Database, AppError> {
        if !self.health.permits_writes() {
            return Err(AppError::NotReady(self.health.clone()));
        }
        self.database.as_ref().ok_or(AppError::NoDatabase)
    }

    pub(crate) fn assess(&self) -> StoreHealth {
        StoreHealth::assess(Assessment {
            sync_folder: self.settings.sync_folder.clone(),
            home_zone_is_set: self.home_zone.is_some(),
        })
    }

    pub(crate) fn take_lock(&mut self) {
        let Some(folder) = self.settings.sync_folder.clone() else {
            return;
        };
        if !self.health.permits_writes() {
            return;
        }
        match WriterLock::acquire(
            &folder,
            AcquireLock { settings: &self.settings, clock: self.clock.as_ref() },
        ) {
            Ok(lock) => self.lock = Some(lock),
            Err(blocked) => self.health = blocked,
        }
    }
}
```

`setup.rs` — the use cases:

```rust
use super::error::AppError;
use super::service::{PlanningApp, StartRequest};
use chrono_tz::Tz;
use planning_store::{
    Database, DeviceSettingsFile, HomeSettingsRepository, SetZone, StoreHealth,
};
use std::path::PathBuf;

impl PlanningApp {
    pub async fn start(request: StartRequest) -> Result<Self, AppError> {
        let settings_file = DeviceSettingsFile::at(request.settings_path);
        let settings = settings_file.load()?;
        let mut app = Self {
            settings_file,
            settings,
            clock: request.clock,
            database: None,
            home_zone: None,
            health: StoreHealth::Unreadable { detail: "not opened".into() },
            lock: None,
        };
        app.reconnect().await?;
        Ok(app)
    }

    pub async fn choose_sync_folder(&mut self, folder: PathBuf) -> Result<StoreHealth, AppError> {
        self.settings.sync_folder = Some(folder);
        self.settings_file.save(&self.settings)?;
        self.reconnect().await?;
        Ok(self.health())
    }

    pub async fn set_home_zone(&mut self, zone: Tz) -> Result<StoreHealth, AppError> {
        let database = self.database.as_ref().ok_or(AppError::NoDatabase)?;
        HomeSettingsRepository::set_zone(
            database,
            SetZone { zone, clock: self.clock.as_ref() },
        )
        .await?;
        self.home_zone = Some(zone);
        self.health = self.assess();
        self.take_lock();
        Ok(self.health())
    }

    /// Re-runs the whole open sequence. Called at start, after choosing a folder,
    /// and by plan 0008 when synchronization recovers.
    pub async fn reconnect(&mut self) -> Result<StoreHealth, AppError> {
        self.lock = None;
        self.database = None;
        self.home_zone = None;

        let Some(folder) = self.settings.sync_folder.clone() else {
            self.health = self.assess();
            return Ok(self.health());
        };
        if folder.is_dir() {
            let database = Database::open(&folder).await?;
            self.home_zone = HomeSettingsRepository::load(&database).await?.home_zone;
            self.database = Some(database);
        }
        self.health = self.assess();
        self.take_lock();
        Ok(self.health())
    }
}
```

`lib.rs`:

```rust
//! The application API. The desktop binary and the launcher depend on this crate
//! and on nothing beneath it.

mod private;

pub use planning_core::{CalendarWeek, Clock, FixedClock, HomeCalendar, SystemClock};
pub use planning_store::{DeviceSettings, SetupGap, StoreHealth};
pub use private::error::AppError;
pub use private::service::{PlanningApp, StartRequest};
```

Re-exporting the types the binaries need is what keeps the boundary test in plan 0002 satisfiable:
`src-tauri` names `planning_app::StoreHealth`, never `planning_store::StoreHealth`.

- [ ] **Step 5: Run the tests to verify they pass**

```bash
cargo test -p planning-app
```

Expected: PASS — 3 tests.

- [ ] **Step 6: Expose the commands from Tauri**

Replace `src-tauri/src/private/commands.rs` (keep `app_version`) and add:

```rust
use planning_app::{AppError, PlanningApp, StoreHealth};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Managed state. A Mutex, not a RwLock: setup mutates, and the app is
/// single-user by construction.
pub struct AppState(pub Arc<Mutex<PlanningApp>>);

fn to_message(error: AppError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn store_health(state: tauri::State<'_, AppState>) -> Result<StoreHealth, String> {
    Ok(state.0.lock().await.health())
}

#[tauri::command]
pub async fn choose_sync_folder(
    state: tauri::State<'_, AppState>,
    folder: PathBuf,
) -> Result<StoreHealth, String> {
    state.0.lock().await.choose_sync_folder(folder).await.map_err(to_message)
}

#[tauri::command]
pub async fn set_home_zone(
    state: tauri::State<'_, AppState>,
    zone: String,
) -> Result<StoreHealth, String> {
    let parsed = zone.parse().map_err(|_| format!("'{zone}' is not an IANA time zone"))?;
    state.0.lock().await.set_home_zone(parsed).await.map_err(to_message)
}
```

Add `planning-app = { workspace = true }`, `tokio = { version = "1", features = ["sync"] }`, and
`chrono-tz = { workspace = true }` to `src-tauri/Cargo.toml`. In `src-tauri/src/lib.rs`, build the
`PlanningApp` with `SystemClock` and `DeviceSettingsFile::default_path()` before
`tauri::Builder`, register it with `.manage(AppState(...))`, and extend `generate_handler!` with
the three new commands.

- [ ] **Step 7: Extend the frontend API module**

Add to `src/lib/api/index.ts`:

```ts
export type SetupGap = 'noSyncFolder' | 'noHomeZone';

export type StoreHealth =
  | { status: 'ready' }
  | { status: 'setupIncomplete'; reason: { kind: 'NoSyncFolder' | 'NoHomeZone' } }
  | { status: 'folderMissing'; path: string }
  | { status: 'lockedByAnotherDevice'; deviceName: string; since: string }
  | { status: 'syncConflict'; artifacts: string[] }
  | { status: 'unreadable'; detail: string };

export function storeHealth(): Promise<StoreHealth> {
  return call<StoreHealth>('store_health');
}

export function chooseSyncFolder(folder: string): Promise<StoreHealth> {
  return call<StoreHealth>('choose_sync_folder', { folder });
}

export function setHomeZone(zone: string): Promise<StoreHealth> {
  return call<StoreHealth>('set_home_zone', { zone });
}
```

Add matching cases to `src/lib/api/index.test.ts` following the `appVersion` pattern from plan
0002 — mock `invoke`, assert the command name and the argument object.

**Serde naming check:** `StoreHealth` uses `#[serde(tag = "status", rename_all = "camelCase")]`,
so variants arrive as `"ready"`, `"setupIncomplete"`, etc., while the nested `SetupGap` uses
`#[serde(tag = "kind")]` without renaming, so its values stay `"NoSyncFolder"` / `"NoHomeZone"`.
Add a Rust test asserting the exact JSON so the TypeScript type cannot drift:

```rust
#[test]
fn store_health_serializes_as_the_frontend_expects() {
    let json = serde_json::to_string(&StoreHealth::SetupIncomplete {
        reason: SetupGap::NoHomeZone,
    })
    .unwrap();
    assert_eq!(json, r#"{"status":"setupIncomplete","reason":{"kind":"NoHomeZone"}}"#);
}
```

Put it in `crates/planning-store/src/private/health.rs` and add `serde_json` to that crate's
dev-dependencies.

- [ ] **Step 8: Run the full gate**

```bash
npm run check && fallow audit
```

Expected: PASS. The boundary test from plan 0002 now has something real to check — confirm it
still passes with `src-tauri` depending on `planning-app` only.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock crates/planning-app src-tauri src/lib/api
git commit -m "feat(app): add the narrow application API with setup and health commands"
```

---

### Task 8: Documentation

**Files:**
- Create: `docs/architecture/storage.md`, `docs/flows/opening-the-app.md`,
  `docs/lessons-learned/sync-safety-is-a-value-not-an-exception.md`
- Modify: `docs/architecture/README.md`, `docs/flows/README.md`,
  `docs/lessons-learned/README.md`, `docs/live/current-status.md`, `docs/adr/0001-embedded-surrealdb-with-rocksdb.md`

- [ ] **Step 1: Write `docs/architecture/storage.md`** (target 80 lines)

Cover: the three crates and their one-way dependency direction; the Synchronization Folder layout
(`planning-db/`, `writer.lock`, and — from plan 0006 — `weekly-reports/`); why device settings live
in the OS config directory instead; the `StoreHealth` variants and the rule that only `Ready`
permits writes; the writer lock's cooperative nature, its 15-minute staleness window, and why
deleting a lock file does not violate ADR 0002; the conflict-artifact patterns detected; and that
`HomeCalendar` is the only source of dates.

- [ ] **Step 2: Write `docs/flows/opening-the-app.md`**

Use the format from `docs/flows/README.md`: Trigger (user launches the app) → Entry point
(`src-tauri/src/lib.rs` → `PlanningApp::start`) → Steps (load device settings → open database if a
folder is set → load home zone → assess health → acquire writer lock) → Reads → Writes → Side
effects (creates `planning-db/`, writes `writer.lock`) → Files to inspect → Common failure modes
(folder not yet mounted by Drive, conflict artifacts present, another device holding a fresh lock,
setup never completed).

- [ ] **Step 3: Write the lessons-learned entry**

Topic: modelling sync safety as a returned `StoreHealth` value rather than an error thrown from
each write. What it buys: the UI can render the exact fault, the launcher can make a go/no-go
decision without catching exceptions, and no write path can forget the check because
`require_database` returns `Err` unless health is `Ready`. Include the counter-intuitive part —
the failure the design prevents is a *silent* write against a half-synchronized RocksDB directory,
which no exception type would have made visible.

- [ ] **Step 4: Register all three docs in their README index tables and update `current-status.md`**

- [ ] **Step 5: Amend ADR 0001**

Add a `## Amendments` section recording two implementation decisions this plan locked in: the
cooperative `writer.lock` file with a 15-minute staleness window (ADR 0001 required one active
writer but did not say how it is detected), and the fact that the home time zone starts unset so
that setup is explicitly incomplete rather than silently defaulting to UTC.

- [ ] **Step 6: Commit**

```bash
git add docs
git commit -m "docs: document storage architecture, open-app flow, and sync-safety lesson"
```

---

## Task 9: Verify the plan's own acceptance

- [ ] `cargo test --workspace` passes; `planning-core` has 9 tests, `planning-store` 20,
      `planning-app` 3.
- [ ] `npm run check` and `fallow audit` both pass.
- [ ] Deleting the `sync_folder` value from the device settings file and restarting the app yields
      `SetupIncomplete { NoSyncFolder }` rather than a crash.
- [ ] Creating a file named `CURRENT (1)` inside the Synchronization Folder makes `store_health`
      return `syncConflict` and blocks `calendar()`.
- [ ] Copying the Synchronization Folder to a second location and opening it there returns the same
      home time zone — the A1 storage-layer proof.
- [ ] `tests/architecture.test.ts` still passes: no `Utc::now()` outside `system_clock.rs`, no
      `chrono::Local`, no `surrealdb` dependency in `src-tauri/Cargo.toml`.

**Next:** [0004-planning-domain.md](0004-planning-domain.md).
