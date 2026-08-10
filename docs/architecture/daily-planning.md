# Daily Planning

Daily Plans, Weekly Focus, Habit Check-ins, and Recurring Task materialization in `planning-core` and `planning-app`. Extends the Library domain from [planning-domain.md](planning-domain.md) with time-scoped records and forward-only propagation (ADR 0002).

## Crate direction

```
src-tauri ──► planning-app ──► planning-store ──► planning-core
                  │
                  └── plan_views, materialization, daily_plan_use_cases
```

## Record keys as invariants

Uniqueness is structural — the SurrealDB record key *is* the rule, not something a query must remember:

| Record | Key | Guarantees |
|--------|-----|------------|
| **DailyPlan** | ISO date, `"2026-08-07"` | Exactly one plan per calendar day |
| **WeeklyFocus** | Week label, `"2026-W32"` | Exactly one focus per Calendar Week |
| **HabitCheckIn** | `"{habitId}:{date}"` | One check-in per Habit per day; correcting is an upsert |
| **Occurrence** | `"{ruleId}:{date}"` | A Recurring Task materializes at most once per date |

See [record-keys-as-invariants.md](../lessons-learned/record-keys-as-invariants.md) for why this pays off.

## Daily Plan — ids only

A `DailyPlan` stores ordered `TaskId` and `HabitId` lists — nothing else about those entities. At read time, `plan_view` resolves each id against current entity state and projects:

| Flag | Source |
|------|--------|
| `archived` | `!lifecycle.is_active()` on the resolved Task or Habit |
| `unpinned` | Habit is no longer pinned (habits only) |
| `outcome` | `HabitCheckIn` for that habit on the plan date, if any |
| `overdue`, `TaskState` | Same projections as the Library |

Archiving or unpinning never writes to an existing plan. The entry stays in place, flagged, and still completable. Tomorrow's plan is seeded from *current* state, so archived entities cannot be newly selected (`require_selectable_task`).

## Pinned-and-due seeding

When a plan is **created** for a date, `habits_due_on` collects Habits that are simultaneously:

- `pinned`
- `lifecycle: Active`
- due on that date's weekday (`Cadence::is_due`)

Those ids are stored in the new plan's `habits` list. **Seeding happens once, at creation.**

`open_plan` loads an existing plan and returns it unchanged — it never re-seeds. Re-seeding would rewrite the user's own selections and violate forward-only propagation. Unpinning or changing cadence takes effect from the *next* plan only. A Habit that is not pinned or not due can still be added manually via `add_habit_to_plan`.

`open_today` calls `materialize_due` first (so the Task Pool is complete), then `open_plan(today)`.

## Task Pool membership

`task_pool` includes every active Task that is either open or non–one-off (completed non–one-off Tasks stay poolable). One-off Tasks that are completed or archived are excluded. Weekly Focus ids still sort into `focus` vs `rest`; membership is independent of focus.

Completion UI is **Daily Plan only** — Task Pool and Library rows never show a completion checkbox or completed flag.

## Recurring Task materialization

`RecurringTask` rules are factories. `materialize_due` walks active rules from `first_candidate` through today, creating a `Task` and an `Occurrence` record for each missing due date. The `Occurrence` key makes a second call a no-op (acceptance A5).

### `CATCH_UP_DAYS = 31`

`PlanningApp::CATCH_UP_DAYS` caps how far back materialization catches up after a long absence. Without the cap, reopening after months away would dump hundreds of stale Tasks into the Task Pool. This is a **product decision**, not an optimization — the number is a deliberate trade-off between catching up and flooding the pool.

### Monthly clamping

`Recurrence::MonthlyDay { day }` uses `effective_day`: the wanted day is clamped to the last day of the month. `MonthlyDay { day: 31 }` occurs on 28 February (29 in a leap year), not skipped. Skipping would make a monthly rule silently vanish for five months a year.

## `has_plan_for` is read-only

`has_plan_for(date)` checks whether a `DailyPlan` record exists — it does **not** create one. Plan 0008's launcher depends on this: asking "does today have a plan?" must never materialize a plan as a side effect.

## Application API — `planning-app`

| Area | Module | Examples |
|------|--------|----------|
| Open plan | `daily_plan_use_cases.rs` | `open_today`, `open_plan`, `has_plan_for` |
| Plan editing | `daily_plan_use_cases.rs` | `select_into_plan`, `reorder_plan`, `remove_from_plan`, `add_habit_to_plan` |
| Check-ins | `check_in_use_cases.rs` | `record_check_in` |
| Weekly Focus | `weekly_focus_use_cases.rs` | `add_to_focus`, `remove_from_focus` |
| Materialization | `materialization.rs` | `materialize_due`, `create_recurring_task` |
| Read models | `plan_views.rs` | `today_view`, `plan_view`, `task_pool` |

All writes go through `PlanningApp::require_database()` — refused unless `StoreHealth` is `Ready`.

## Frontend mirror

TypeScript types in `src/lib/domain/index.ts` match Rust JSON-shape tests on view structs. Tauri commands in `src-tauri/src/private/plan_commands.rs` delegate to `planning-app` only.
