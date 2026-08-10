# Completing a Task from the Daily Plan

## Trigger

User clicks **Mark done** on a task row in today's plan (`PlanTaskRow` / `TaskCompletionToggle`).

## Entry point

`DailyPlanStore.toggleCompletion(task)` in `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts`

## Steps — complete (open → completed)

1. **UI** — `PlanTaskRow` **Mark done** button calls `activeStore.toggleCompletion(task)`.
2. **Store** — `task.state !== 'completed'`, so the store calls `api.completeTask(task.id)`.
3. **IPC** — `src/lib/api/index.ts` → `call('complete_task', { task })` → Tauri `complete_task`.
4. **Command** — `src-tauri/src/private/lifecycle_commands.rs` locks `AppState` and calls `PlanningApp::complete_task`.
5. **Domain** — `entity_lifecycle.rs` resolves **today** in the home time zone (`calendar().today(clock)`), then mutates the Task: `completion = Completed { on: today }`. Lifecycle is untouched.
6. **Reload** — store `#change` awaits the mutation, then `load()` fetches fresh `todayView()` + `taskPool()`.
7. **Reprojection** — `plan_view` resolves stored task ids; sets `state: 'completed'`, `archived`, `overdue`, and position flags at read time. UI re-renders with `StateFlag kind="completed"` and muted row styling.

No write to `daily_plan` — the plan still lists the task id; only the Task entity's `completion` axis changes.

## Steps — reopen (completed → open)

Same path, but `toggleCompletion` sees `task.state === 'completed'` and calls `api.reopenTask(task.id)` → `PlanningApp::reopen_task` → `completion = Open`. Reload and reprojection follow.

Completion is reversible at any time (ADR 0002). Reopening is not gated on plan membership.

## Archived entry — the surprising case

A task archived from the Library **stays in today's plan** (forward-only propagation). On the next `today_view`, `plan_view` sets `archived: true` on the projected row. **Mark done** stays enabled — completion is **never** gated on `lifecycle`.

User ticks an archived open task → same `complete_task` path. The Task becomes `archived` **and** `completed` (orthogonal axes). Both `StateFlag` kinds can appear. Restoring from the Library later returns it completed, not open.

Proven by `DailyPlan.test.ts` ("still lets an archived entry be completed") and `entity_lifecycle.rs` tests.

## Reads

| Source | What |
|--------|------|
| `daily_plan` table | Ordered task ids for the date (unchanged) |
| `task` table | `completion` and `lifecycle` for projection |
| Home settings | Today's date for `Completed { on }` stamp |

## Writes

| Target | What changes |
|--------|--------------|
| `task` record | `completion` only (`Completed { on }` or `Open`) |

## Side effects

- Task Pool: one-off completed Tasks drop out on next `task_pool()`; non–one-off completed Tasks stay poolable while active.
- Weekly Focus read models reflect the new `state` on next load.
- Weekly summary counts the completion on the stamped date, not necessarily the plan date.

Completion is toggled from the Daily Plan for every task; the Library also offers completion controls for **one-off** tasks only.

## Files to inspect

| Path | Role |
|------|------|
| `src/lib/surfaces/daily-plan/Private/PlanTaskRow.svelte` | Mark done trigger |
| `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts` | `toggleCompletion`, `#change`, `load` |
| `src/lib/api/index.ts` | `completeTask`, `reopenTask` |
| `src-tauri/src/private/lifecycle_commands.rs` | IPC handlers |
| `crates/planning-app/src/private/entity_lifecycle.rs` | `complete_task`, `reopen_task` |
| `crates/planning-app/src/private/plan_views.rs` | `today_view`, `plan_view` projection |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| Checkbox toggles then snaps back | IPC error; `#change` catches and leaves prior view (check `store.error`) |
| Task not in plan | Id not in `daily_plan.tasks` — completion still works from the Daily Plan row when the task is listed there |
| Cannot complete archived task | Bug — completion must not check `lifecycle` |
| Completed one-off missing from pool | Expected — only non–one-off completed Tasks stay poolable |
