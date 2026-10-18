# Completing a Task from the Daily Plan

## Trigger

User clicks the round check on a task row in Today (`PlanTaskRow` → `TaskLine`). The right-click **Done and archive** variant is in [right-click-task-actions.md](right-click-task-actions.md).

## Entry point

`DailyPlanStore.toggleCompletion(task, date)` in `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts`

## Steps — complete (open → done)

1. **UI** — `TaskLine` check (`role="checkbox"`, label "Mark done: …") calls `activeStore.toggleCompletion(task, planView.date)`.
2. **Store** — `task.completed` is false, so the store calls `api.completeTask(task.id, date)`.
3. **IPC** — `call('complete_task', { task, on })` → `lifecycle_commands.rs::complete_task` → `complete_task_on` (or `complete_task` without a date).
4. **App** — `task_completion.rs` refuses dates after home-zone today (`FutureCompletion`), then `mutate` applies `Task::complete(on)`.
5. **Domain** — `planning-core/task_outcome.rs`: `completion = Completed { on }`; **one-off Tasks also get `lifecycle = Archived`** in the same write. Ongoing (non-one-off) Tasks keep their lifecycle.
6. **Reload** — `#change` → `load()` fetches `todayView`, `taskPool`, `yesterdayView`, then `weeklyFocus(plan.week)`.
7. **Reprojection** — `PlanTaskView.completed: true` (set from the completion axis even though `state` is now `archived`). The row stays in Today, struck through, until the day ends.

No write to `daily_plan` — the plan still lists the id.

## Steps — reopen (done → open)

`toggleCompletion` sees `task.completed` and calls `api.reopenTask` → `Task::reopen()`: `completion = Open`, and a one-off returns to `Active`. Reversible any time (ADR 0002).

## Archived entry — the surprising cases

- A task archived from the Library **stays in today's plan** (forward-only). The check stays enabled — completion is never gated on `lifecycle`.
- Because done one-offs archive themselves, `state === 'archived'` on a plan row usually means *done*, not *dropped*. Read `completed`, never `state`, to decide done-ness.
- Reopening an ongoing task the user archived does **not** restore it; only one-offs come back.

## Reads / Writes

| Target | Read | Write |
|--------|------|-------|
| `daily_plan` | ordered ids | — |
| `task` | completion, lifecycle, one_off | `completion`; `lifecycle` for one-offs |

## Side effects

- Library: a done one-off leaves the Tasks tab and appears under **Archive → Done** with its date.
- Task Pool: done one-offs drop out (archived); done ongoing tasks stay poolable while active.
- Weekly summary counts the completion on the stamped date regardless of lifecycle.

## Files to inspect

| Path | Role |
|------|------|
| `src/lib/surfaces/daily-plan/Private/PlanTaskRow.svelte` | Row + check |
| `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts` | `toggleCompletion`, `#change`, `load` |
| `src-tauri/src/private/lifecycle_commands.rs` | IPC handlers |
| `crates/planning-app/src/private/task_completion.rs` | `complete_task(_on)`, `reopen_task`, `rename_task` |
| `crates/planning-core/src/private/task_outcome.rs` | auto-archive rule |
| `crates/planning-app/src/private/plan_views.rs` | `PlanTaskView.completed` projection |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| Check toggles then snaps back | IPC error; `#change` keeps the prior view (see `store.error`) |
| Done row shows as not done | Code reading `state === 'completed'` — archived wins in `state`; use `completed` / `completedOn` |
| Cannot complete archived task | Bug — completion must not check `lifecycle` |
| Done one-off missing from Library Tasks tab | Expected — it is in the Archive tab |
