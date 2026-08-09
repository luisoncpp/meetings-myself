# Creating a Recurring Task

## Trigger

User clicks **New recurring task** in the Library surface (`RecurringTaskSection`).

## Entry point

`RecurringTaskSection.handleCreate` → `LibraryStore.createRecurringTask` in `src/lib/surfaces/library/Private/LibraryStore.svelte.ts`

## Steps

1. **UI** — `RecurringTaskSection` sets `creating = true`; `CreateRecurringTask` shows title field + `CreateRecurrence` picker. Submit is enabled when title is non-empty and recurrence is valid.
2. **Form submit** — `CreateRecurringTask` calls `oncreate({ title, recurrence })` → `handleCreate` → `store.createRecurringTask`.
3. **Store** — `#change` awaits `api.createRecurringTask(title, recurrence)` (pessimistic; no local rule list update before IPC succeeds).
4. **IPC** — `src/lib/api/index.ts` → `call('create_recurring_task', …)` → Tauri `create_recurring_task`.
5. **Command** — `src-tauri/src/private/plan_commands.rs` locks `AppState` and calls `PlanningApp::create_recurring_task`.
6. **Domain** — `materialization.rs`: resolve **today** in the home time zone as `starts_on`; `RecurringTask::create` with `lifecycle: Active`, `materialized_through: None`; persist to `recurring_task` table. **No** `materialize_due` on this path.
7. **Reload** — `#change` calls `load()`, which parallel-fetches `library()`, `todayView()`, and `recurringTasks()`; the new rule appears in `RecurringTaskSection`. `creating` resets to false.

**Later — first materialization** (not part of this user action):

8. Next `open_today` → `materialize_due()` walks active rules from `first_candidate` through today (see [opening-todays-plan.md](opening-todays-plan.md)).
9. For each due date without an existing `Occurrence`, `materialize_one` creates a `Task` (title copied from the rule at materialization time) and an `Occurrence` keyed `{ruleId}:{date}`.
10. Produced Tasks appear in the Task Pool only — **not** auto-selected into `daily_plan.tasks`.

Renaming the rule later updates the rule record only; already-materialized Tasks keep their original title (ADR 0002). Archiving stops future materialization; past occurrences remain.

## Reads

| Source | What |
|--------|------|
| Home settings | Today's date for `starts_on` |
| `recurring_task` table | Full rule list on reload (`recurring_tasks`) |
| `library` view | Association entities (parallel fetch; rules excluded) |

## Writes

| Target | What changes |
|--------|--------------|
| `recurring_task` record | New rule (`title`, `recurrence`, `starts_on`, `lifecycle: Active`) |

No `task`, `occurrence`, or `daily_plan` write during create.

## Side effects

- None until the next `open_today` / `materialize_due` (see [opening-todays-plan.md](opening-todays-plan.md)).
- After materialization: new Tasks in Task Pool; user must `select_into_plan` to add them to today's plan.

## Files to inspect

| Path | Role |
|------|------|
| `src/lib/surfaces/library/Private/RecurringTaskSection.svelte` | Section shell, create toggle |
| `src/lib/surfaces/library/Private/CreateRecurringTask.svelte` | Create form |
| `src/lib/surfaces/library/Private/CreateRecurrence.svelte` | Recurrence picker + validation |
| `src/lib/surfaces/library/Private/RecurringTaskRow.svelte` | List row (rename, archive) |
| `src/lib/surfaces/library/Private/LibraryStore.svelte.ts` | `createRecurringTask`, `load`, `#change` |
| `src/lib/api/index.ts` | `createRecurringTask`, `recurringTasks` |
| `src-tauri/src/private/plan_commands.rs` | IPC handlers |
| `crates/planning-app/src/private/materialization.rs` | `create_recurring_task`, `materialize_due` |
| `crates/planning-core/src/private/recurring_task.rs` | `RecurringTask::create`, `Occurrence::key` |
| `docs/flows/opening-todays-plan.md` | Materialize step on plan open |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| Create button stays disabled | Empty title or invalid recurrence in `CreateRecurrence` |
| Form submits then rule missing | IPC error; `#change` sets `store.error` and skips optimistic update |
| Command error "not ready" | Health gate / `require_database` — setup incomplete or store not ready |
| `starts_on` / calendar errors | Home zone not set |
| Rule visible but no Task yet | Expected — create does not materialize; wait for `open_today` / `materialize_due` |
| Task in pool but not on plan | Expected — materialization does not `select_into_plan` |
| Weekly rule created mid-week, no Task today | `occurs_on(today)` false — first Task appears on the next matching weekday |
| Duplicate Tasks after reopen | Should not happen — check `Occurrence` key guard (A5) |
