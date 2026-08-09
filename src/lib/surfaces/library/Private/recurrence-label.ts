import type { Recurrence, Weekday } from '../../../domain';
import { WEEKDAYS } from './labels';

export function recurrenceLabel(recurrence: Recurrence): string {
  if (recurrence.kind === 'daily') return 'Daily';
  if (recurrence.kind === 'weekdays') return 'Weekdays';
  if (recurrence.kind === 'weekly') {
    const weekday = weekdayLabel(recurrence.weekday);
    return `Weekly · ${weekday}`;
  }
  return `Monthly · day ${recurrence.day}`;
}

function weekdayLabel(weekday: Weekday): string {
  const match = WEEKDAYS.find((day) => day.value === weekday);
  return match?.label ?? weekday;
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
