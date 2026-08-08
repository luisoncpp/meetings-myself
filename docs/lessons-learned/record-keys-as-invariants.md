# Record Keys as Invariants

**Date:** 2026-08-08

## What looks intuitive

Duplicate prevention needs bookkeeping: a transaction, a "find existing" query, a `last_run` timestamp, or a mutex around materialization. Reopening the app is a natural place to accidentally create the same occurrence twice.

## What we did

Choose the SurrealDB record key so it **is** the uniqueness rule:

| Record | Key |
|--------|-----|
| Daily Plan | the date (`"2026-08-07"`) |
| Weekly Focus | the week label (`"2026-W32"`) |
| Habit Check-in | `habitId:date` |
| Occurrence | `ruleId:date` |

`Records::save` upserts by `(table, key)`. A second write with the same key replaces the document instead of creating a sibling.

## Payoff

- **A5 (no duplicate occurrences):** `materialize_due` can run on every app open. If the `Occurrence` key already exists, `materialize_one` skips. No transaction, no pre-query race, no `last_run` as the source of truth.
- **One plan per day:** `open_plan` loads by date key; create only when `find` returns `None`.
- **Correcting check-ins:** recording again for the same habit on the same day is an upsert, not find-then-update.
- **`has_plan_for`:** a cheap existence check with no side effects — safe for the launcher.

## The counter-intuitive part

`RecurringTask.materialized_through` looks like the correctness mechanism — "we already processed through this date." It is not. It is a **performance hint** only: `first_candidate` uses it to avoid scanning from `starts_on` on every open.

Correctness comes entirely from the `Occurrence` key. If `materialized_through` were wrong, stale, or reset, the worst case is redundant work — walking dates that already have occurrences — not duplicate Tasks. Treating `materialized_through` as authoritative is exactly the bug this design avoids.

## Generalizable lesson

When the natural identity of a thing is already unique (a date, a week, a habit-on-a-day), put that identity in the **record key** and let the storage upsert enforce it. Keep auxiliary fields as hints for efficiency, never as substitutes for structural uniqueness.
