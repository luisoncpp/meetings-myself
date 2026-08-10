import type { Recurrence, Weekday } from '../../../domain';
import { t } from '../../../i18n';
import { weekdayLabel } from './labels';

export function recurrenceLabel(recurrence: Recurrence): string {
  if (recurrence.kind === 'daily') return t('domain.recurrence.daily');
  if (recurrence.kind === 'weekdays') return t('domain.recurrence.weekdays');
  if (recurrence.kind === 'weekly') {
    return t('domain.recurrence.weekly', { weekday: weekdayLabel(recurrence.weekday) });
  }
  return t('domain.recurrence.monthly', { day: recurrence.day });
}

export function buildRecurrence(
  kind: Recurrence['kind'],
  weekday: Weekday,
  monthlyDay: string,
): Recurrence {
  if (kind === 'daily') return { kind: 'daily' };
  if (kind === 'weekdays') return { kind: 'weekdays' };
  if (kind === 'weekly') return { kind: 'weekly', weekday };
  return { kind: 'monthlyDay', day: Number(monthlyDay) };
}

export function isRecurrenceValid(
  kind: Recurrence['kind'],
  weekday: Weekday,
  monthlyDay: string,
): boolean {
  if (kind === 'weekly' && !weekday) return false;
  if (kind !== 'monthlyDay') return true;
  const day = Number(monthlyDay);
  return Number.isInteger(day) && day >= 1 && day <= 31;
}
