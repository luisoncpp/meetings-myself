# Daily Plan Launcher — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](0001-self-planning-app.md) first. Requires
> [0005-daily-plan-and-habits.md](done/0005-daily-plan-and-habits.md). Task 5 (settings UI) additionally
> benefits from [0007-ui-surfaces.md](done/0007-ui-surfaces.md) but does not require it.

**Goal:** A separate background binary that opens the app at 7:00 AM home time *only* when no Daily
Plan exists for today and the synchronized data is safe, retries through a configurable morning
window after startup or sync recovery, and records a missed prompt when the window closes.

**Architecture:** The scheduling logic is a **pure function** — `decide(schedule, situation) ->
Decision` — with no clock, no IO, and no async. The daemon loop is a thin shell that gathers a
`Situation`, calls `decide`, and performs the returned `Decision`. Every rule in this plan,
including "never launch from unsafe data", is therefore an ordinary unit test with no timers and no
sleeping.

The launcher opens the store **read-only**: it never acquires the writer lock, so having the
launcher running can never block the app or another device.

**Tech Stack:** Rust 1.95, `planning-app` only, `auto-launch` 0.6 for login registration, `tokio`
for the loop.

---

## Global constraints

See [0001-self-planning-app.md](0001-self-planning-app.md#global-constraints). Load-bearing here:

- **`launcher/Cargo.toml` may depend on `planning-app` and nothing beneath it** — enforced by
  `tests/architecture.test.ts` from plan 0002, which currently passes vacuously because the crate
  does not exist. It stops being vacuous in Task 1.
- **No `Utc::now()` outside `system_clock.rs`; no `chrono::Local`.** The launcher is the most
  tempting place to break this and the worst place to do it — a device in a different zone would
  fire at the wrong hour.
- **The launcher never writes to the synchronized database.** It reads, and it writes only the
  device-local settings file.

### Decision recorded in plan 0001, implemented here

ADR 0001 asks for a launcher built on a read-only API, and also asks it to record a missed prompt.
Those conflict unless the record is device-local. **The missed prompt is written to the device
settings file** (`last_missed_prompt`, already defined in plan 0003), not to the synchronized
database. It is a device fact: a second machine that was switched off did not miss anything.
Task 6 records this as an ADR 0001 amendment.

---

## File structure

| File | Responsibility |
|------|----------------|
| `launcher/Cargo.toml` | Binary manifest — `planning-app` only |
| `launcher/src/main.rs` | Entry point; calls `run()` |
| `launcher/src/lib.rs` | Public interface |
| `launcher/src/private/schedule.rs` | `LauncherSchedule`, window arithmetic |
| `launcher/src/private/decision.rs` | `Situation`, `Decision`, `decide` — **pure** |
| `launcher/src/private/daemon.rs` | The loop that performs decisions |
| `launcher/src/private/app_process.rs` | Spawning the desktop app |
| `crates/planning-app/src/private/read_only.rs` | `PlanningApp::start_read_only` |
| `src-tauri/src/private/autostart_commands.rs` | Enable/disable login registration |
| `src/lib/surfaces/settings/` | Launcher settings UI |
| `docs/architecture/launcher.md` | New architecture doc |
| `docs/flows/morning-launch.md` | New flow doc |

---

### Task 1: Read-only start

**Files:**
- Create: `crates/planning-app/src/private/read_only.rs`
- Modify: `crates/planning-app/src/private/service.rs`, `lib.rs`
- Test: inline in `read_only.rs`

**Interfaces:**
- Consumes: `PlanningApp::start` from plan 0003.
- Produces:
  - `PlanningApp::start_read_only(StartRequest) -> Result<PlanningApp, AppError>` — identical to
    `start` except it never calls `take_lock`.
  - `PlanningApp::is_read_only(&self) -> bool`
  - Every write path returns `AppError::ReadOnly` when the app was started read-only.

Without that last guard, `start_read_only` would be a comment rather than a constraint. `store` and
`mutate` (plan 0004) are the only write paths, so one check in each covers everything.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_support::ready_folders;
    use planning_store::WriterLock;

    #[tokio::test]
    async fn a_read_only_app_reads_but_never_writes() {
        let (home, drive) = ready_folders().await;
        let app = PlanningApp::start_read_only(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 6, 0, 0).unwrap())),
        })
        .await
        .unwrap();

        assert!(app.is_read_only());
        assert_eq!(app.health(), StoreHealth::Ready);
        assert!(app.tasks().await.is_ok(), "reads work");
        assert!(matches!(
            app.create_task("Nope".into()).await.unwrap_err(),
            AppError::ReadOnly
        ));
    }

    #[tokio::test]
    async fn a_read_only_app_never_takes_the_writer_lock() {
        let (home, drive) = ready_folders().await;
        let _launcher = PlanningApp::start_read_only(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 6, 0, 0).unwrap())),
        })
        .await
        .unwrap();

        assert!(
            !drive.path().join(WriterLock::FILE_NAME).exists(),
            "the launcher must never block the app or another device"
        );
    }

    #[tokio::test]
    async fn has_plan_for_still_works_read_only() {
        let (home, _drive) = ready_folders().await;
        let app = PlanningApp::start_read_only(StartRequest {
            settings_path: home.path().join("device-settings.json"),
            clock: Arc::new(FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 6, 0, 0).unwrap())),
        })
        .await
        .unwrap();
        let today = app.calendar().unwrap().today(app.clock_ref());
        assert!(!app.has_plan_for(today).await.unwrap());
    }
}
```

`ready_folders` is a `test_support` helper that creates the two temp dirs, runs a normal
`PlanningApp` through setup (folder + home zone), drops it so the lock is released, and returns the
two `TempDir`s.

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-app read_only
```

Expected: FAIL — `no function 'start_read_only'`.

Add `pub(crate) read_only: bool` to `PlanningApp`, set it in both constructors, guard `take_lock`
with an early return, and add to `store` and `mutate`:

```rust
if self.read_only {
    return Err(AppError::ReadOnly);
}
```

Add `#[error("this instance opened the data read-only")] ReadOnly` to `AppError`.

Factor the shared construction so `start` and `start_read_only` are two three-line functions over
one private `start_with(request, read_only)` — the boolean is internal, so no named-parameter
comment is needed at the definition, but both public call sites read
`Self::start_with(request, /*readOnly=*/true)`.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS.

```bash
git add crates/planning-app
git commit -m "feat(app): add read-only start for the launcher"
```

---

### Task 2: The scheduling decision

**Files:**
- Create: `launcher/Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/private/mod.rs`,
  `src/private/schedule.rs`, `src/private/decision.rs`
- Modify: root `Cargo.toml` (workspace members)

**Interfaces:**
- Consumes: `planning_app::{DeviceSettings, StoreHealth}` and chrono types.
- Produces:

```rust
pub struct LauncherSchedule { pub launch_time: NaiveTime, pub retry_window_minutes: u32, pub zone: Tz }
impl LauncherSchedule {
    pub fn from(settings: &DeviceSettings, zone: Tz) -> Self;
    pub fn window_start(&self, day: NaiveDate) -> DateTime<Utc>;
    pub fn window_end(&self, day: NaiveDate) -> DateTime<Utc>;
    pub fn today(&self, now: DateTime<Utc>) -> NaiveDate;
}

pub struct Situation {
    pub now: DateTime<Utc>,
    pub store_is_ready: bool,
    pub plan_exists: bool,
    pub missed_prompt_recorded_for: Option<NaiveDate>,
}

pub enum Decision {
    Sleep { seconds: u64 },
    LaunchApp,
    RecordMissedPrompt { date: NaiveDate },
}

pub fn decide(schedule: &LauncherSchedule, situation: &Situation) -> Decision;
pub const RETRY_SECONDS: u64 = 300;
```

**This task claims acceptance criterion A7.** `decide` returns `LaunchApp` only when
`store_is_ready` is true — there is exactly one place that decision is made, and it is a pure
function with a test named after the criterion.

The rules, in order:

| Situation | Decision |
|-----------|----------|
| before today's window | `Sleep` until `window_start` |
| in the window, a plan already exists | `Sleep` until tomorrow's `window_start` |
| in the window, store is **not** ready | `Sleep { RETRY_SECONDS }` — never launch |
| in the window, store ready, no plan | `LaunchApp` |
| after the window, a plan exists | `Sleep` until tomorrow's `window_start` |
| after the window, missed prompt already recorded for today | `Sleep` until tomorrow |
| after the window, nothing recorded | `RecordMissedPrompt { today }` |

DST is why `window_start` is computed by projecting the home-zone date and time back to UTC on
every evaluation rather than adding 24 hours to a stored instant. On a spring-forward day the gap
between consecutive windows is 23 hours, and arithmetic on instants would drift.

- [ ] **Step 1: Create the crate**

Add `launcher` to workspace members.

```toml
[package]
name = "planning-launcher"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[[bin]]
name = "planning-launcher"
path = "src/main.rs"

[dependencies]
planning-app = { workspace = true }
chrono = { workspace = true }
chrono-tz = { workspace = true }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }

[dev-dependencies]
tempfile = "3.27.0"
```

Note there is **no** `planning-store`, `planning-core`, or `surrealdb` here. Plan 0002's boundary
test now has a real second binary to check.

- [ ] **Step 2: Write the failing decision tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono_tz::Tz;

    /// 07:00 Madrid with a four-hour window, so the window is 05:00-09:00 UTC in August.
    fn schedule() -> LauncherSchedule {
        LauncherSchedule {
            launch_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            retry_window_minutes: 240,
            zone: Tz::Europe__Madrid,
        }
    }

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 7, hour, minute, 0).unwrap()
    }

    fn situation(now: DateTime<Utc>) -> Situation {
        Situation { now, store_is_ready: true, plan_exists: false, missed_prompt_recorded_for: None }
    }

    #[test]
    fn before_the_window_it_sleeps_until_the_window_opens() {
        let decision = decide(&schedule(), &situation(at(3, 0)));
        // 03:00 UTC to 05:00 UTC is two hours.
        assert!(matches!(decision, Decision::Sleep { seconds } if seconds == 7200));
    }

    #[test]
    fn in_the_window_with_no_plan_and_healthy_data_it_launches() {
        assert!(matches!(decide(&schedule(), &situation(at(5, 0))), Decision::LaunchApp));
        assert!(matches!(decide(&schedule(), &situation(at(8, 59))), Decision::LaunchApp));
    }

    /// Acceptance criterion A7.
    #[test]
    fn it_never_launches_from_unavailable_or_unsafe_data() {
        let blocked = Situation { store_is_ready: false, ..situation(at(6, 0)) };
        assert!(matches!(
            decide(&schedule(), &blocked),
            Decision::Sleep { seconds } if seconds == RETRY_SECONDS
        ));
    }

    #[test]
    fn an_existing_plan_means_nothing_to_do_until_tomorrow() {
        let done = Situation { plan_exists: true, ..situation(at(6, 0)) };
        let Decision::Sleep { seconds } = decide(&schedule(), &done) else {
            panic!("expected Sleep");
        };
        // 06:00 UTC today to 05:00 UTC tomorrow is 23 hours.
        assert_eq!(seconds, 23 * 3600);
    }

    #[test]
    fn after_the_window_it_records_a_missed_prompt_exactly_once() {
        let missed = decide(&schedule(), &situation(at(9, 30)));
        let Decision::RecordMissedPrompt { date } = missed else {
            panic!("expected RecordMissedPrompt, got {missed:?}");
        };
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 8, 7).unwrap());

        let already = Situation {
            missed_prompt_recorded_for: Some(date),
            ..situation(at(9, 31))
        };
        assert!(matches!(decide(&schedule(), &already), Decision::Sleep { .. }));
    }

    #[test]
    fn a_late_start_inside_the_window_still_launches() {
        // The computer was switched on at 08:30 UTC; the window closes at 09:00.
        assert!(matches!(decide(&schedule(), &situation(at(8, 30))), Decision::LaunchApp));
    }

    #[test]
    fn the_window_follows_the_home_zone_not_utc() {
        let los_angeles = LauncherSchedule { zone: Tz::America__Los_Angeles, ..schedule() };
        // 07:00 in Los Angeles is 14:00 UTC in August, so 05:00 UTC is far too early.
        assert!(matches!(decide(&los_angeles, &situation(at(5, 0))), Decision::Sleep { .. }));
        assert!(matches!(decide(&los_angeles, &situation(at(14, 30))), Decision::LaunchApp));
    }

    #[test]
    fn windows_are_recomputed_per_day_so_dst_does_not_drift() {
        // 29 March 2026 is the spring-forward day in Madrid: 02:00 becomes 03:00.
        let day_before = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();
        let day_after = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
        let gap = schedule().window_start(day_after) - schedule().window_start(day_before);
        assert_eq!(gap.num_hours(), 23, "07:00 local stays 07:00 local across the change");
    }
}
```

- [ ] **Step 3: Run to verify they fail, then implement `schedule.rs`**

```bash
cargo test -p planning-launcher
```

```rust
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use planning_app::DeviceSettings;

/// When the launcher may open the app, expressed in the synchronized home zone.
#[derive(Clone, Copy, Debug)]
pub struct LauncherSchedule {
    pub launch_time: NaiveTime,
    pub retry_window_minutes: u32,
    pub zone: Tz,
}

impl LauncherSchedule {
    pub fn from(settings: &DeviceSettings, zone: Tz) -> Self {
        Self {
            launch_time: settings.launch_time,
            retry_window_minutes: settings.retry_window_minutes,
            zone,
        }
    }

    pub fn today(&self, now: DateTime<Utc>) -> NaiveDate {
        now.with_timezone(&self.zone).date_naive()
    }

    /// Projected fresh from the home-zone date every time. Adding 24 hours to the
    /// previous window would drift by an hour on each DST change.
    pub fn window_start(&self, day: NaiveDate) -> DateTime<Utc> {
        let local = day.and_time(self.launch_time);
        self.zone
            .from_local_datetime(&local)
            // A spring-forward gap can make 07:00 not exist in some zones; the
            // earliest valid instant afterwards is the honest answer.
            .earliest()
            .unwrap_or_else(|| self.zone.from_utc_datetime(&local))
            .with_timezone(&Utc)
    }

    pub fn window_end(&self, day: NaiveDate) -> DateTime<Utc> {
        self.window_start(day) + Duration::minutes(i64::from(self.retry_window_minutes))
    }
}
```

- [ ] **Step 4: Implement `decision.rs`**

```rust
use super::schedule::LauncherSchedule;
use chrono::{DateTime, Duration, NaiveDate, Utc};

/// How long to wait before re-checking when the store is not ready. Google Drive
/// can take minutes to finish synchronizing after login.
pub const RETRY_SECONDS: u64 = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Situation {
    pub now: DateTime<Utc>,
    pub store_is_ready: bool,
    pub plan_exists: bool,
    pub missed_prompt_recorded_for: Option<NaiveDate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Sleep { seconds: u64 },
    LaunchApp,
    RecordMissedPrompt { date: NaiveDate },
}

/// The whole scheduling policy, as a pure function. No clock, no IO, no async —
/// which is what lets every rule below be an ordinary unit test.
pub fn decide(schedule: &LauncherSchedule, situation: &Situation) -> Decision {
    let today = schedule.today(situation.now);
    let start = schedule.window_start(today);

    if situation.now < start {
        return sleep_until(situation.now, start);
    }
    if situation.plan_exists {
        return sleep_until_tomorrow(schedule, situation.now, today);
    }
    if situation.now <= schedule.window_end(today) {
        // A7: unsafe or unavailable data never opens the app. Wait it out.
        if !situation.store_is_ready {
            return Decision::Sleep { seconds: RETRY_SECONDS };
        }
        return Decision::LaunchApp;
    }
    if situation.missed_prompt_recorded_for == Some(today) {
        return sleep_until_tomorrow(schedule, situation.now, today);
    }
    Decision::RecordMissedPrompt { date: today }
}

fn sleep_until_tomorrow(
    schedule: &LauncherSchedule,
    now: DateTime<Utc>,
    today: NaiveDate,
) -> Decision {
    sleep_until(now, schedule.window_start(today + Duration::days(1)))
}

fn sleep_until(now: DateTime<Utc>, target: DateTime<Utc>) -> Decision {
    let seconds = (target - now).num_seconds().max(1);
    Decision::Sleep { seconds: seconds as u64 }
}
```

`sleep_until_tomorrow` takes three parameters, which is the limit.

- [ ] **Step 5: Run, commit**

```bash
cargo test -p planning-launcher
```

Expected: PASS — 8 tests. **A7 is now proven.**

```bash
git add Cargo.toml Cargo.lock launcher
git commit -m "feat(launcher): add the pure scheduling decision"
```

---

### Task 3: Spawning the app

**Files:**
- Create: `launcher/src/private/app_process.rs`
- Test: inline

**Interfaces:**
- Produces:
  - `app_executable() -> Result<PathBuf, LauncherError>` — the desktop binary beside the launcher.
  - `launch(path: &Path) -> Result<(), LauncherError>` — spawns it detached; does **not** wait.
  - `LauncherError` (thiserror): `App(#[from] AppError)`, `NotFound { path: PathBuf }`,
    `Spawn(#[from] std::io::Error)`, `NoHomeZone`.

`app_executable` resolves from `std::env::current_exe()`'s directory so a portable install works,
with the file name `Self-Planning.exe` on Windows and `Self-Planning` elsewhere. It returns
`NotFound` rather than spawning a guess.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn a_missing_executable_is_reported_rather_than_guessed_at() {
        let empty = TempDir::new().unwrap();
        let error = executable_in(empty.path()).unwrap_err();
        assert!(matches!(error, LauncherError::NotFound { .. }));
    }

    #[test]
    fn the_app_is_found_beside_the_launcher() {
        let folder = TempDir::new().unwrap();
        let path = folder.path().join(APP_FILE_NAME);
        std::fs::write(&path, "").unwrap();
        assert_eq!(executable_in(folder.path()).unwrap(), path);
    }
}
```

- [ ] **Step 2: Run to verify it fails, implement, run, commit**

```rust
use super::error::LauncherError;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
pub const APP_FILE_NAME: &str = "Self-Planning.exe";
#[cfg(not(windows))]
pub const APP_FILE_NAME: &str = "Self-Planning";

/// The desktop app ships beside the launcher, so resolve relatively — a portable
/// install must work without an absolute path anywhere.
pub fn app_executable() -> Result<PathBuf, LauncherError> {
    let current = std::env::current_exe()?;
    let folder = current.parent().unwrap_or(Path::new("."));
    executable_in(folder)
}

pub fn executable_in(folder: &Path) -> Result<PathBuf, LauncherError> {
    let candidate = folder.join(APP_FILE_NAME);
    if !candidate.exists() {
        return Err(LauncherError::NotFound { path: candidate });
    }
    Ok(candidate)
}

/// Starts the app and returns immediately. The launcher must not become the
/// app's parent-in-waiting — it has its own loop to get back to.
pub fn launch(path: &Path) -> Result<(), LauncherError> {
    Command::new(path).spawn()?;
    Ok(())
}
```

```bash
cargo test -p planning-launcher
git add launcher
git commit -m "feat(launcher): resolve and spawn the desktop app"
```

---

### Task 4: The daemon loop

**Files:**
- Create: `launcher/src/private/daemon.rs`, `launcher/src/private/error.rs`,
  `launcher/src/lib.rs`, `launcher/src/main.rs`
- Test: inline in `daemon.rs`

**Interfaces:**
- Produces:
  - `Daemon::new(DaemonSetup { app: PlanningApp, settings_file: DeviceSettingsFile }) -> Self`
  - `Daemon::tick(&mut self) -> Result<Decision, LauncherError>` — gathers a `Situation`, calls
    `decide`, performs it, and returns what it did. **Never sleeps** — the caller does.
  - `Daemon::run(mut self) -> Result<(), LauncherError>` — the loop: `tick`, then
    `tokio::time::sleep` on a `Sleep` decision.
  - `run() -> Result<(), LauncherError>` in `lib.rs` — wires `PlanningApp::start_read_only`, the
    device settings file, and `Daemon::run`.

Splitting `tick` from `run` is what makes the loop testable: every test drives `tick` with a
`FixedClock` it advances by hand, and no test sleeps.

Each `tick` calls `app.reconnect()` before assessing health. That single line is what "retries after
computer startup or synchronization recovery" means in practice — a folder that was not yet mounted
at login becomes `Ready` on a later tick with no special-case code.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    /// Ready store, no plan, clock at 06:00 UTC — inside the 05:00-09:00 window.
    async fn daemon_at(hour: u32) -> (TempDir, TempDir, Daemon, Arc<FixedClock>) { /* ... */ }

    #[tokio::test]
    async fn it_launches_once_then_stands_down_for_the_day() {
        let (_home, _drive, mut daemon, clock) = daemon_at(6).await;

        assert_eq!(daemon.tick().await.unwrap(), Decision::LaunchApp);
        assert_eq!(daemon.launch_attempts(), 1);

        // The app has now created today's plan.
        daemon.mark_plan_created_for_test();
        clock.advance(Duration::minutes(10));
        assert!(matches!(daemon.tick().await.unwrap(), Decision::Sleep { .. }));
        assert_eq!(daemon.launch_attempts(), 1, "never twice in a day");
    }

    #[tokio::test]
    async fn it_waits_out_an_unready_store_and_launches_when_sync_recovers() {
        let (_home, drive, mut daemon, clock) = daemon_at(6).await;
        std::fs::write(drive.path().join("CURRENT (1)"), "").unwrap();

        assert_eq!(
            daemon.tick().await.unwrap(),
            Decision::Sleep { seconds: RETRY_SECONDS },
            "A7: a conflict must not open the app"
        );
        assert_eq!(daemon.launch_attempts(), 0);

        std::fs::remove_file(drive.path().join("CURRENT (1)")).unwrap();
        clock.advance(Duration::seconds(RETRY_SECONDS as i64));
        assert_eq!(daemon.tick().await.unwrap(), Decision::LaunchApp);
    }

    #[tokio::test]
    async fn a_missed_prompt_is_written_to_the_device_settings_file_only() {
        let (home, drive, mut daemon, _clock) = daemon_at(10).await; // after the window
        let before: Vec<_> = std::fs::read_dir(drive.path()).unwrap().collect();

        let decision = daemon.tick().await.unwrap();
        assert!(matches!(decision, Decision::RecordMissedPrompt { .. }));

        let settings = DeviceSettingsFile::at(home.path().join("device-settings.json"))
            .load()
            .unwrap();
        assert_eq!(settings.last_missed_prompt, Some(NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()));

        let after: Vec<_> = std::fs::read_dir(drive.path()).unwrap().collect();
        assert_eq!(
            after.len(),
            before.len(),
            "the synchronized folder must be untouched by the launcher"
        );
    }

    #[tokio::test]
    async fn it_reports_rather_than_dies_when_the_app_binary_is_absent() {
        let (_home, _drive, mut daemon, _clock) = daemon_at(6).await;
        // `daemon_at` points the daemon at an empty folder for the executable.
        let error = daemon.tick().await.unwrap_err();
        assert!(matches!(error, LauncherError::NotFound { .. }));
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement `daemon.rs`**

```bash
cargo test -p planning-launcher daemon
```

```rust
use super::app_process;
use super::decision::{decide, Decision, Situation};
use super::error::LauncherError;
use super::schedule::LauncherSchedule;
use planning_app::{DeviceSettingsFile, PlanningApp};

pub struct DaemonSetup {
    pub app: PlanningApp,
    pub settings_file: DeviceSettingsFile,
}

pub struct Daemon {
    app: PlanningApp,
    settings_file: DeviceSettingsFile,
    launch_attempts: u32,
}

impl Daemon {
    pub fn new(setup: DaemonSetup) -> Self {
        Self { app: setup.app, settings_file: setup.settings_file, launch_attempts: 0 }
    }

    pub fn launch_attempts(&self) -> u32 {
        self.launch_attempts
    }

    /// One evaluation. Never sleeps — `run` owns the waiting so tests can drive
    /// this directly with a pinned clock.
    pub async fn tick(&mut self) -> Result<Decision, LauncherError> {
        // Re-opening the store is how "retry after startup or sync recovery"
        // works: a folder Drive had not yet mounted becomes Ready on a later tick.
        self.app.reconnect().await?;

        let schedule = self.schedule()?;
        let situation = self.situation(&schedule).await?;
        let decision = decide(&schedule, &situation);
        self.perform(&decision)?;
        Ok(decision)
    }

    pub async fn run(mut self) -> Result<(), LauncherError> {
        loop {
            let decision = self.tick().await?;
            let Decision::Sleep { seconds } = decision else {
                // A launch or a recorded prompt: check back shortly to update state.
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                continue;
            };
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        }
    }

    fn perform(&mut self, decision: &Decision) -> Result<(), LauncherError> {
        match decision {
            Decision::Sleep { .. } => Ok(()),
            Decision::LaunchApp => {
                let executable = app_process::app_executable()?;
                app_process::launch(&executable)?;
                self.launch_attempts += 1;
                Ok(())
            }
            Decision::RecordMissedPrompt { date } => {
                // Device-local only. A machine that was switched off missed nothing,
                // so this must never travel through the Synchronization Folder.
                let mut settings = self.settings_file.load()?;
                settings.last_missed_prompt = Some(*date);
                self.settings_file.save(&settings)?;
                Ok(())
            }
        }
    }
}
```

`schedule()` reads the device settings and the home zone from `self.app.calendar()?.zone()`,
returning `LauncherError::NoHomeZone` when setup is incomplete — a launcher with no home zone must
not guess an hour. `situation()` assembles `now` from the app's clock, `store_is_ready` from
`self.app.health().permits_writes()`, `plan_exists` from `has_plan_for(today)`, and
`missed_prompt_recorded_for` from the settings file.

`lib.rs`:

```rust
//! The Daily Plan Launcher. Opens the desktop app on a morning schedule, but only
//! when no plan exists yet and the synchronized data is safe to read (ADR 0001).

mod private;

pub use private::daemon::{Daemon, DaemonSetup};
pub use private::decision::{decide, Decision, Situation, RETRY_SECONDS};
pub use private::error::LauncherError;
pub use private::schedule::LauncherSchedule;

use planning_app::{DeviceSettingsFile, PlanningApp, StartRequest, SystemClock};
use std::sync::Arc;

pub async fn run() -> Result<(), LauncherError> {
    let settings_path = DeviceSettingsFile::default_path()?;
    let app = PlanningApp::start_read_only(StartRequest {
        settings_path: settings_path.clone(),
        clock: Arc::new(SystemClock),
    })
    .await?;
    Daemon::new(DaemonSetup { app, settings_file: DeviceSettingsFile::at(settings_path) })
        .run()
        .await
}
```

`DeviceSettingsFile::default_path` must be re-exported from `planning-app` — add it if plan 0003 did
not.

`main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    if let Err(error) = planning_launcher::run().await {
        eprintln!("planning-launcher stopped: {error}");
        std::process::exit(1);
    }
}
```

`windows_subsystem = "windows"` keeps a console window from flashing at every login.

- [ ] **Step 3: Run the full workspace and commit**

```bash
cargo test --workspace && npm run check
```

Expected: PASS, including plan 0002's boundary test now checking a real `launcher/Cargo.toml`.

```bash
git add launcher
git commit -m "feat(launcher): add the daemon loop with read-only sync-safe checks"
```

---

### Task 5: Autostart registration and settings UI

**Files:**
- Create: `src-tauri/src/private/autostart_commands.rs`, `src/lib/surfaces/settings/`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src/lib/api/index.ts`,
  `src/lib/shell/Private/Navigation.svelte`

**Interfaces:**
- Produces:
  - Rust commands `launcher_autostart_enabled() -> bool`,
    `set_launcher_autostart(enabled: bool) -> Result<(), String>`,
    `launcher_settings() -> LauncherSettings`,
    `set_launcher_settings(LauncherSettings) -> Result<(), String>`,
    `missed_prompt() -> Option<NaiveDate>`
  - `LauncherSettings { launch_time: String, retry_window_minutes: u32 }` — a projection of the
    device settings, so the UI never touches `device_id` or `sync_folder`.
  - A Settings surface with: enable-at-login toggle, launch time, retry window (in hours), and the
    Synchronization Folder path shown read-only.

`auto-launch` 0.6 registers an **arbitrary** executable, which is what this needs — the thing
registered at login is `planning-launcher`, not the app. `tauri-plugin-autostart` registers the
current executable and would register the wrong binary.

- [ ] **Step 1: Write the failing Rust test for the settings projection**

```rust
#[test]
fn launcher_settings_round_trip_without_exposing_device_identity() {
    let settings = DeviceSettings {
        launch_time: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
        retry_window_minutes: 120,
        ..DeviceSettings::default()
    };
    let projected = LauncherSettings::from(&settings);
    assert_eq!(projected.launch_time, "06:30");
    assert_eq!(projected.retry_window_minutes, 120);

    let mut applied = DeviceSettings::default();
    projected.apply_to(&mut applied).unwrap();
    assert_eq!(applied.launch_time, settings.launch_time);
    assert_ne!(applied.device_id, "", "identity is preserved, not overwritten");
}

#[test]
fn an_unparseable_launch_time_is_refused() {
    let projected = LauncherSettings { launch_time: "25:99".into(), retry_window_minutes: 60 };
    assert!(projected.apply_to(&mut DeviceSettings::default()).is_err());
}
```

- [ ] **Step 2: Implement, wire the autostart commands, build the surface**

```rust
use auto_launch::AutoLaunchBuilder;

const LAUNCHER_FILE_NAME: &str = if cfg!(windows) { "planning-launcher.exe" } else { "planning-launcher" };

/// Registers the LAUNCHER at login, not this app. The launcher decides whether the
/// app should open at all (ADR 0001).
fn launcher_autostart() -> Result<auto_launch::AutoLaunch, String> {
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let launcher = current
        .parent()
        .ok_or("could not resolve the install directory")?
        .join(LAUNCHER_FILE_NAME);
    AutoLaunchBuilder::new()
        .set_app_name("Self-Planning Daily Plan Launcher")
        .set_app_path(&launcher.to_string_lossy())
        .build()
        .map_err(|error| error.to_string())
}
```

The Settings surface follows the `DailyPlanStore` pattern. Copy is explanatory: the toggle says the
launcher opens the app in the morning **only if you have not planned yet**, and the retry window
says how long it keeps trying if the computer was off or Drive was still syncing.

The missed-prompt notice belongs on the Daily Plan surface: a quiet, dismissible line reading
"No plan was made on <date>." — no guilt, no streak break, consistent with PRODUCT.md.

- [ ] **Step 3: Verify by hand**

Enable the toggle, sign out and back in, and confirm the launcher process runs and the app does not
open when a plan already exists. Set the launch time two minutes ahead, delete today's plan record,
and confirm the app opens. Record the result in the architecture doc.

- [ ] **Step 4: Run the gate and commit**

```bash
npm run check && fallow audit
```

```bash
git add src-tauri src/lib
git commit -m "feat(launcher): add login registration and the settings surface"
```

---

### Task 6: Documentation and the ADR amendment

**Files:**
- Create: `docs/architecture/launcher.md`, `docs/flows/morning-launch.md`,
  `docs/lessons-learned/pure-decision-functions-for-scheduled-work.md`
- Modify: `docs/adr/0001-embedded-surrealdb-with-rocksdb.md`, the three README index tables,
  `docs/live/current-status.md`

- [ ] **Step 1: Write `docs/architecture/launcher.md`** (target 70 lines)

Cover: that the launcher is a separate binary depending on `planning-app` only; the read-only start
and why it never takes the writer lock; the full decision table from Task 2; the DST reasoning for
recomputing the window each day; `RETRY_SECONDS`; that the missed prompt is device-local and why;
that `auto-launch` registers the launcher rather than the app; and the manual verification result
from Task 5.

- [ ] **Step 2: Write `docs/flows/morning-launch.md`**

Trigger (login, or a scheduled tick) → `run` → `start_read_only` → `Daemon::tick` → `reconnect` →
gather `Situation` → `decide` → perform. Reads / Writes (device settings only) / Side effects
(spawns the app process) / Failure modes: home zone unset, Drive folder not yet mounted at login,
conflict artifacts present, app binary missing, computer off through the whole window.

- [ ] **Step 3: Write the lessons-learned entry**

Topic: extracting a scheduler's policy into a pure `decide(schedule, situation) -> Decision`. The
payoff: DST correctness, "never launch from unsafe data", and "record the missed prompt exactly
once" are all ordinary unit tests with no timers, no sleeping, and no flakiness. The
counter-intuitive part: the `Sleep` durations are *returned data*, not a side effect — which is what
lets a test assert "23 hours" across a spring-forward boundary instead of waiting for one.

- [ ] **Step 4: Amend ADR 0001**

Extend the `## Amendments` section added in plan 0003 with the two decisions this plan locked in:
the launcher records its missed prompt in the device settings file rather than the synchronized
database, so ADR 0001's read-only requirement holds literally; and the launcher never acquires the
writer lock, so running it can never block the app or another device.

- [ ] **Step 5: Update `docs/live/current-status.md`**

The app is now feature-complete against `docs/plans/0001-self-planning-app.md`. Move every
subsystem to "Implemented", and per `docs/UPDATE.md` move all eight plan files to
`docs/plans/done/`, keeping `0001-self-planning-app.md` as the index with its links updated.

- [ ] **Step 6: Commit**

```bash
git add docs
git commit -m "docs: document the launcher, amend ADR 0001, and close the plan set"
```

---

## Task 7: Verify the plan's own acceptance

- [ ] `cargo test --workspace`, `npm run check`, and `fallow audit` all pass.
- [ ] **A7:** `decide` returns `LaunchApp` only when `store_is_ready` — proven by the Task 2 test,
      and end-to-end by the Task 4 conflict-artifact test.
- [ ] `launcher/Cargo.toml` names only `planning-app`, `chrono`, `chrono-tz`, and `tokio` — plan
      0002's boundary test now checks a real file.
- [ ] Running the launcher leaves no `writer.lock` in the Synchronization Folder.
- [ ] Recording a missed prompt writes to the device settings file and adds no file to the
      Synchronization Folder.
- [ ] The launch window follows the home time zone, not the device zone.
- [ ] With the app already opened and a plan created, the launcher does not open it a second time.

---

## Whole-app verification

With all eight plans complete, confirm every acceptance criterion from
[0001-self-planning-app.md](0001-self-planning-app.md#acceptance-criteria-for-the-whole-app):

| # | Criterion | Where proven |
|---|-----------|--------------|
| A1 | Cross-device visibility after Drive sync | 0003 Task 4; end-to-end by copying the folder |
| A2 | Select, order, remove, complete without duplication | 0005 Task 5 |
| A3 | Pinned habits on cadence days, three outcomes | 0005 Tasks 6–7 |
| A4 | Prior report, one file per week, next week's focus | 0006 Task 5 |
| A5 | Recurring Tasks never duplicate on reopen | 0005 Task 3 |
| A6 | Archived entries stay in place, marked and completable | 0004 Task 9 + 0005 Task 5 |
| A7 | Launcher never opens from unsafe data | 0008 Tasks 2 and 4 |
| A8 | The Weekly Review has no exclusive powers | 0006 Task 5 + 0007 Task 7 |

Then do one manual end-to-end pass on two machines sharing a Drive folder: create a Task on the
first, close the app, wait for Drive to finish, and confirm it appears on the second — the only
part of A1 that no automated test can reach.
