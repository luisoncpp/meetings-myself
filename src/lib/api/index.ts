import { call } from './Private/bridge';
import type {
  Association,
  AssociationEnd,
  Cadence,
  Classification,
  HabitStrength,
  LibraryView,
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
