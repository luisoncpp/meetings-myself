import { t } from '../../i18n';
import type { EntityKind } from './associations';

export function entityNameLabel(kind: EntityKind): string {
  if (kind === 'habit') return t('createEntity.habitName');
  if (kind === 'goal') return t('createEntity.goalName');
  if (kind === 'task') return t('createEntity.taskName');
  return t('createEntity.valueName');
}

export function createButtonLabel(kind: EntityKind): string {
  if (kind === 'habit') return t('createEntity.createHabit');
  if (kind === 'goal') return t('createEntity.createGoal');
  if (kind === 'task') return t('createEntity.createTask');
  return t('createEntity.createValue');
}
