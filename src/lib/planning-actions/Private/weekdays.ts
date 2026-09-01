import type { Weekday } from '../../domain';
import { t } from '../../i18n';

export function weekdayLabel(value: Weekday): string {
  return t(`domain.weekday.${value}`);
}

export const WEEKDAY_VALUES: readonly Weekday[] = [
  'mon',
  'tue',
  'wed',
  'thu',
  'fri',
  'sat',
  'sun',
];
