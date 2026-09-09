# Catching Up Yesterday's Plan

## Trigger

The Daily Plan surface loads. The user expands **Yesterday** (Ayer) below today's habits and marks leftover tasks done or records habit check-ins.

## Entry point

`DailyPlanStore.load()` → `api.yesterdayView()` → Tauri `yesterday_view` → `PlanningApp::yesterday_view`

Mutations: `toggleCompletion(task, yesterday.date)` and `checkIn(habitId, outcome, yesterday.date)`.

## Steps — load

1. **UI** — `DailyPlan` constructs `DailyPlanStore` and calls `load()`.
2. **Store** — fetches `todayView()`, `taskPool()`, and `yesterdayView()` in parallel.
3. **Yesterday view** — `yesterday_view` takes home-zone today minus one day, then `open_plan` + project. Missing plans are created and seeded.
4. **UI** — `YesterdayCard` always renders **below** today's plan as a collapsed header button. The user expands it, then completes / checks in.

## Steps — complete a leftover task

Same as [completing-a-task-from-the-daily-plan.md](completing-a-task-from-the-daily-plan.md), with `on` = yesterday. `complete_task_on` stamps `Completed { on: yesterday }`. Reload refreshes today and yesterday together.

## Steps — habit check-in

Same `record_check_in` path as today, with `date` = yesterday. Key `{habitId}:{date}` upserts that day only; today's check-in is unchanged.

## Reads

| Source | What |
|--------|------|
| Home settings | Today in the home zone, to compute yesterday |
| `daily_plan` | Yesterday's id lists, if the record exists |
| `task`, `habit`, `habit_check_in` | Projection |

## Writes

| Target | What changes |
|--------|--------------|
| `daily_plan` | Created and seeded if yesterday had no plan |
| `task` | `completion` when toggling a leftover (`on` = yesterday) |
| `habit_check_in` | Upsert for `(habit, yesterday)` |

Opening today's surface may create yesterday's plan. It still does not create extra days beyond yesterday.

## Side effects

- Weekly summary counts the task on the stamped date (yesterday).
- Habit tallies for that week include the corrected check-in.
- A Task on both plans shows completed on both rows (one completion axis).

## Files to inspect

| Path | Role |
|------|------|
| `src/lib/surfaces/daily-plan/Private/YesterdayCard.svelte` | Catch-up card |
| `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts` | Load + dated mutations |
| `src/lib/api/index.ts` | `yesterdayView`, `completeTask(task, on?)` |
| `src-tauri/src/private/plan_commands.rs` | `yesterday_view` |
| `crates/planning-app/src/private/plan_views.rs` | `yesterday_view` |
| `crates/planning-app/src/private/entity_lifecycle.rs` | `complete_task_on` |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| No Yesterday card | `yesterdayView` failed (check `store.error`); header is below habits and starts collapsed |
| Yesterday created on first catch-up | Expected when that day was never opened |
| Leftover counted as today in Weekly Review | Completion used `complete_task` without `on` |
| FutureCompletion error | `on` after home-zone today |
