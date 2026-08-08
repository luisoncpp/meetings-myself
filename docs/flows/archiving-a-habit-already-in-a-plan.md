# Archiving a Habit Already in a Plan

## Trigger

User archives a Habit from the Library while that Habit is already listed in today's (or any past) Daily Plan.

This is the flow that answers **"why is this still here?"** — the most surprising behavior in the app, and deliberate (ADR 0002 / forward-only propagation).

## Entry point

`src/lib/api/index.ts` → `archiveEntity({ kind: 'habit', id })` → Tauri `archive_entity` → `PlanningApp::archive_habit`

See [archiving-an-entity.md](archiving-an-entity.md) for the generic archive path.

## Steps

1. **IPC** — frontend passes `{ kind: 'habit', id }`.
2. **Load** — `Records::find` on the `habit` table; missing id → `NotFound`.
3. **Mutate** — set `lifecycle` to `Archived`. Cadence, `pinned`, and strength are **not** changed.
4. **Save** — `Records::save` upserts the Habit document.
5. **Plan unchanged** — no write to `daily_plan`, `habit_check_in`, or `association` tables.
6. **Read model** — next `today_view` / `plan_view` resolves the Habit id still stored in the plan; `project_plan_habits` sets `archived: true`. Check-in recording still works; existing outcomes still display.

The Habit cannot be newly seeded into tomorrow's plan (`lifecycle` is not active) and cannot be manually added via `add_habit_to_plan` on a fresh day. It **can** still be checked in on the day it already appears.

## Reads

| Source | What |
|--------|------|
| `habit` table | Current Habit document |
| `daily_plan` table | Unchanged — still lists the Habit id |
| `habit_check_in` table | Outcome for projection, if recorded |

## Writes

| Target | What changes |
|--------|--------------|
| Habit record | `lifecycle: Archived` only |

## Side effects

**None** on plans, check-ins, or associations.

Specifically, archiving does **not**:

- Remove the Habit id from any `DailyPlan`
- Delete or invalidate existing `HabitCheckIn` records
- Archive linked Tasks, Goals, or Values
- Change `pinned` or cadence (those affect *future* seeding only)
- Block recording or correcting a check-in for that Habit on a day it is already in the plan

Restore (`restore_entity`) flips `lifecycle` back to `Active`; the plan still lists the id; `archived` clears on the next projection.

## How it renders

`PlanHabitView` for an archived Habit already in the plan:

| Field | Value |
|-------|-------|
| `archived` | `true` |
| `unpinned` | current pinned state (orthogonal) |
| `outcome` | `Done`, `Skipped`, `NotCompleted`, or `null` |
| Check-in actions | Still allowed — outcomes stay correctable (ADR 0002) |

The entry remains visible and ordered with the rest of the plan's habits. The UI should show the archived flag honestly rather than hiding the row.

## Files to inspect

| Path | Role |
|------|------|
| `src-tauri/src/private/lifecycle_commands.rs` | `archive_entity` |
| `crates/planning-app/src/private/habit_lifecycle.rs` | `archive_habit` |
| `crates/planning-app/src/private/plan_projection.rs` | `archived: !habit.lifecycle.is_active()` |
| `crates/planning-app/src/private/daily_plan_use_cases.rs` | `habits_due_on` filters active only |
| `crates/planning-app/src/private/check_in_use_cases.rs` | Check-in not gated on lifecycle |
| `docs/flows/archiving-an-entity.md` | Generic archive flow |

## Common failure modes

| Symptom | Cause |
|---------|-------|
| Habit vanished from today's plan | Bug — projection must not drop stored ids; check `project_plan_habits` |
| Habit still auto-appears tomorrow after archive | Bug — `habits_due_on` should filter `lifecycle.is_active()` |
| Cannot check in archived Habit on today's plan | Bug — completion/check-in must not be gated on archive state |
| User expects archive to remove from plan | Product misunderstanding — forward-only is intentional; see ADR 0002 |
