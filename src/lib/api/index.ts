import { call } from './Private/bridge';
import type {
  Association,
  AssociationEnd,
  Cadence,
  CheckInOutcome,
  Classification,
  DailyPlanView,
  HabitStrength,
  LibraryView,
  Recurrence,
  RecurringTask,
  TaskPoolView,
  WeeklyFocus,
} from '../domain';

export function appVersion(): Promise<string> {
  return call<string>('app_version');
}

export type StoreHealth =
  | { status: 'ready' }
  | { status: 'setupIncomplete'; reason: { kind: 'NoSyncFolder' | 'NoHomeZone' } }
  | { status: 'folderMissing'; path: string }
  | { status: 'lockedByAnotherDevice'; deviceName: string; since: string }
  | { status: 'syncConflict'; artifacts: string[] }
  | { status: 'unreadable'; detail: string };

export function storeHealth(): Promise<StoreHealth> {
  return call<StoreHealth>('store_health');
}

export function chooseSyncFolder(folder: string): Promise<StoreHealth> {
  return call<StoreHealth>('choose_sync_folder', { folder });
}

export function setHomeZone(zone: string): Promise<StoreHealth> {
  return call<StoreHealth>('set_home_zone', { zone });
}

export function library(includeArchived: boolean): Promise<LibraryView> {
  return call<LibraryView>('library', { includeArchived });
}

export function createValue(title: string): Promise<unknown> {
  return call('create_value', { title });
}

export function createTask(title: string): Promise<unknown> {
  return call('create_task', { title });
}

export function createGoal(title: string, targetDate?: string | null): Promise<unknown> {
  return call('create_goal', { title, targetDate: targetDate ?? null });
}

export function createHabit(title: string, cadence: Cadence): Promise<unknown> {
  return call('create_habit', { title, cadence });
}

export function archiveEntity(end: AssociationEnd): Promise<void> {
  return call<void>('archive_entity', { end });
}

export function restoreEntity(end: AssociationEnd): Promise<void> {
  return call<void>('restore_entity', { end });
}

export function completeTask(task: string): Promise<void> {
  return call<void>('complete_task', { task });
}

export function reopenTask(task: string): Promise<void> {
  return call<void>('reopen_task', { task });
}

export function achieveGoal(goal: string): Promise<void> {
  return call<void>('achieve_goal', { goal });
}

export function unachieveGoal(goal: string): Promise<void> {
  return call<void>('unachieve_goal', { goal });
}

export function classifyTask(
  task: string,
  importance: Classification,
  urgency: Classification,
): Promise<void> {
  return call<void>('classify_task', { task, importance, urgency });
}

export function setTaskDeadline(task: string, deadline?: string | null): Promise<void> {
  return call<void>('set_task_deadline', { task, deadline: deadline ?? null });
}

export function setHabitCadence(habit: string, cadence: Cadence): Promise<void> {
  return call<void>('set_habit_cadence', { habit, cadence });
}

export function setHabitPinned(habit: string, pinned: boolean): Promise<void> {
  return call<void>('set_habit_pinned', { habit, pinned });
}

export function setHabitStrength(habit: string, strength: HabitStrength): Promise<void> {
  return call<void>('set_habit_strength', { habit, strength });
}

export function link(left: AssociationEnd, right: AssociationEnd): Promise<Association> {
  return call<Association>('link', { left, right });
}

export function unlink(association: string): Promise<void> {
  return call<void>('unlink', { association });
}

export function associationsFor(end: AssociationEnd): Promise<Association[]> {
  return call<Association[]>('associations_for', { end });
}

export function todayView(): Promise<DailyPlanView> {
  return call<DailyPlanView>('today_view');
}

export function taskPool(): Promise<TaskPoolView> {
  return call<TaskPoolView>('task_pool');
}

export function selectIntoPlan(date: string, task: string): Promise<void> {
  return call<void>('select_into_plan', { date, task });
}

export function removeFromPlan(date: string, task: string): Promise<void> {
  return call<void>('remove_from_plan', { date, task });
}

export function reorderPlan(date: string, order: string[]): Promise<void> {
  return call<void>('reorder_plan', { date, order });
}

export function addHabitToPlan(date: string, habit: string): Promise<void> {
  return call<void>('add_habit_to_plan', { date, habit });
}

export function quickAddTask(title: string): Promise<unknown> {
  return call('quick_add_task', { title });
}

export function recordCheckIn(
  habit: string,
  date: string,
  outcome: CheckInOutcome,
): Promise<void> {
  return call<void>('record_check_in', { habit, date, outcome });
}

export function weeklyFocus(week: string): Promise<WeeklyFocus> {
  return call<WeeklyFocus>('weekly_focus', { week });
}

export function addToFocus(week: string, task: string): Promise<void> {
  return call<void>('add_to_focus', { week, task });
}

export function removeFromFocus(week: string, task: string): Promise<void> {
  return call<void>('remove_from_focus', { week, task });
}

export function createRecurringTask(
  title: string,
  recurrence: Recurrence,
): Promise<RecurringTask> {
  return call<RecurringTask>('create_recurring_task', { title, recurrence });
}

export function recurringTasks(): Promise<RecurringTask[]> {
  return call<RecurringTask[]>('recurring_tasks');
}

export function archiveRecurringTask(rule: string): Promise<void> {
  return call<void>('archive_recurring_task', { rule });
}

export function restoreRecurringTask(rule: string): Promise<void> {
  return call<void>('restore_recurring_task', { rule });
}
