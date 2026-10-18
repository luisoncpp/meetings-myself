# Planning Domain & Library

Values, Goals, Habits, Tasks, and Associations in `planning-core`, persisted through a generic `Records` gateway in `planning-store`, and exposed to the UI through `planning-app` use cases and read-model views.

## Crate direction

```
src-tauri ──► planning-app ──► planning-store ──► planning-core
                  │                  │
                  └──────────────────┘
```

`src-tauri` depends on `planning-app` only. Domain types cross the IPC boundary through re-exports on `planning-app`'s public API.

## Entities

| Entity | Fields | Creation rule |
|--------|--------|---------------|
| **Value** | `id`, `title`, `lifecycle`, `created_at` | Title alone |
| **Goal** | `id`, `title`, `achievement`, `lifecycle`, `target_date?`, `created_at` | Title alone; optional target date |
| **Task** | `id`, `title`, `completion`, `lifecycle`, `importance`, `urgency`, `deadline?`, `one_off`, `created_at` | Title alone; `one_off` defaults to `true`; importance/urgency default to unclassified. Title can change later (`rename_task`, blank rejected) |
| **Habit** | `id`, `title`, `lifecycle`, `strength`, `cadence`, `pinned`, `created_at` | Title **and** cadence required; new habits are pinned and reminder-dependent. Title can change later (`rename_habit`); a blank title is rejected. The id stays, so check-ins remain attached |
| **Association** | `id`, `left`, `right`, `lifecycle`, `created_at` | Valid pair only; stored in canonical order |

### Orthogonal lifecycle

A Task's outcome and its archive state are **independent axes**:

- `Completion`: `Open` | `Completed { on }`
- `Lifecycle`: `Active` | `Archived`

Archiving a completed Task keeps the completion. Restoring it returns it exactly as it was — no guesswork. Goals use the same pattern with `Achievement` × `Lifecycle`.

**One-off auto-archive** (`Task::complete` / `Task::reopen` in `planning-core/task_outcome.rs`): completing a **one-off** Task also sets `Archived`; reopening it sets `Active`. The axes stay orthogonal in storage — the rule only couples the *transitions*. Ongoing (non-one-off) Tasks never change lifecycle on complete/reopen, and reopening never restores an ongoing Task the user archived. Recurring occurrences are one-off Tasks, so a done occurrence archives; the rule keeps producing new ones.

`TaskState` (`Open` | `Completed` | `Archived`) exists only in **read models** for display. Archived wins when both axes apply.

### Associations

Supported pairs (canonical order: Value < Goal < Habit < Task):

| Left | Right |
|------|-------|
| Value | Goal |
| Goal | Habit |
| Goal | Task |
| Habit | Task |

Archiving one end never cascades. Links stay in the database; `associations_for` returns active links only.

## Persistence — `Records`

One generic gateway in `planning-store` — no per-entity repositories, **no delete path** (ADR 0002).

| Method | Role |
|--------|------|
| `Records::save` | Upsert by `(table, id)` |
| `Records::find` | Single record or `None` |
| `Records::all` | All records in a table |

### SurrealDB 3 adaptation

SurrealDB 3's upsert/select APIs require `SurrealValue`, not plain `Serialize` domain types. `Records` round-trips through `serde_json::Value` while keeping generic `Serialize`/`Deserialize` bounds at the public API. On save, the document `id` field is stripped (the record key is authoritative). On load, SurrealDB record IDs are normalized back to bare UUID strings for id newtypes.

## Application API — `planning-app`

| Area | Module | Examples |
|------|--------|----------|
| Creation | `library.rs` | `create_task`, `create_habit` |
| Lifecycle | `entity_lifecycle.rs`, `habit_lifecycle.rs`, `task_completion.rs` | `archive_task`, `set_task_one_off`, `set_habit_cadence`, `rename_habit`; `complete_task`, `complete_task_on`, `reopen_task`, `rename_task` |
| Associations | `associations.rs` | `link`, `unlink` (archives link) |
| Read models | `views.rs`, `views_entities.rs` | `library(LibraryFilter)` |

All writes go through `PlanningApp::require_database()` — refused unless `StoreHealth` is `Ready`.

## Projections (never stored)

Computed at read time in view types; forward-only for plan 0005:

| Projection | Source |
|------------|--------|
| `overdue` | Open + active Task with `deadline < today` (home zone) |
| `archived` | `!lifecycle.is_active()` |
| `TaskState` | Collapse of `Completion` × `Lifecycle` (archived wins) |
| `TaskView.completed_on` / `PlanTaskView.completed` | The completion axis on its own — read this, not `state`, to know if a task is done |
| `achieved` | `achievement.is_achieved()` |

`LibraryFilter { include_archived }` hides archived entries by default.

## Frontend mirror

TypeScript types in `src/lib/domain/index.ts` match Rust JSON-shape tests on view structs. Tauri commands in `src-tauri/src/private/library_commands.rs` and `lifecycle_commands.rs` delegate to `planning-app` only.
