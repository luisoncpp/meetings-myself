# Opening Today's Plan

## Trigger

The app opens and the Daily Plan surface loads, or the calendar date rolls over while the app is running and the UI requests today's plan again.

## Entry point

`src/lib/api/index.ts` → `todayView()` → Tauri `today_view` → `PlanningApp::today_view`

(`open_today` is the write path when only the domain record is needed; `today_view` calls `open_plan` then `plan_view`.)

## Steps

1. **Require ready store** — `require_database()` refuses if `StoreHealth` is not `Ready`.
2. **Resolve today** — `calendar().today(clock)` in the home time zone; errors if home zone is unset.
3. **Materialize recurring tasks** — `materialize_due()` walks active `RecurringTask` rules and creates any missing `Occurrence` + `Task` pairs up to today (capped by `CATCH_UP_DAYS`).
4. **Load or create plan** — `open_plan(today)`:
   - If a `DailyPlan` record exists for the date key, return it **unchanged**.
   - Otherwise, compute pinned-and-due Habits, call `DailyPlan::start`, and persist.
5. **Project** — `plan_view` resolves every stored id against current Tasks, Habits, and Check-ins; sets `archived`, `unpinned`, `outcome`, and task state flags at read time.

When the clock advances past midnight, the next `open_today` / `today_view` uses a new date key and creates a **new** plan; yesterday's record is untouched.

## Reads

| Source | What |
|--------|------|
| Home settings | `today` date in home zone |
| `recurring_task` table | Active rules and `materialized_through` hint |
| `occurrence` table | Existing materializations (duplicate guard) |
| `daily_plan` table | Plan for the date key, if any |
| `habit` table | Pinned/active/due filter on first create only |
| `task`, `habit`, `habit_check_in` tables | Projection of stored ids |

## Writes

| Target | When |
|--------|------|
| `occurrence` + `task` | `materialize_due` finds a missing due date |
| `recurring_task` | `materialized_through` updated after a rule walk |
| `daily_plan` | First open of a calendar day only (create + seed habits) |

Re-opening the same day writes **nothing** — existing plan is returned as stored.

## Side effects

- New Recurring Task occurrences appear in the Task Pool (not auto-selected into the plan).
- First open of a day seeds pinned-and-due Habits into the plan's `habits` list.
- `has_plan_for` is **not** called on this path; it is read-only for the launcher (plan 0008).

## Files to inspect

| Path | Role |
|------|------|
| `src-tauri/src/private/plan_commands.rs` | `today_view` IPC |
| `crates/planning-app/src/private/daily_plan_use_cases.rs` | `open_today`, `open_plan`, `has_plan_for` |
| `crates/planning-app/src/private/materialization.rs` | `materialize_due`, `CATCH_UP_DAYS` |
| `crates/planning-app/src/private/plan_views.rs` | `today_view`, `plan_view` |
| `crates/planning-app/src/private/plan_projection.rs` | Id → view resolution |
| `crates/planning-core/src/private/daily_plan.rs` | `DailyPlan::key`, `start` |
| `src/lib/api/index.ts` | Frontend `todayView` |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| Command error "not ready" | Setup incomplete, sync conflict, or writer lock held — `require_database` refused |
| `calendar()` / today errors despite `ready` | Home zone not set (`setupIncomplete / NoHomeZone`) |
| Pinned Habit missing from today's plan | Opened before create on a prior day, then unpinned — forward-only; or not a cadence day for a *new* plan |
| Pinned Habit still on plan after unpin | Expected — today's plan is not rewritten; view shows `unpinned: true` |
| Duplicate recurring Tasks after reopen | Should not happen — indicates `Occurrence` key bypass; check A5 |
| Midnight crossed mid-session | UI still showing yesterday until it refetches; next `open_today` creates a new date's plan |
| Hundreds of tasks after long absence | Capped at `CATCH_UP_DAYS + 1` per rule — if more appear, check cap logic |
