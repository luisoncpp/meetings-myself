import type { EntityKind } from './associations';

export function entityNameLabel(kind: EntityKind): string {
  if (kind === 'habit') return 'Habit name';
  if (kind === 'goal') return 'Goal name';
  if (kind === 'task') return 'Task name';
  return 'Value name';
}

export function createButtonLabel(kind: EntityKind): string {
  if (kind === 'habit') return 'Create habit';
  if (kind === 'goal') return 'Create goal';
  if (kind === 'task') return 'Create task';
  return 'Create value';
}
