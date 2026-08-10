import type { Weekday } from '../../domain';
import { t } from '../../i18n';

function weekdayLabel(value: Weekday): string {
  return t(`domain.weekday.${value}`);
}

export const WEEKDAYS: { value: Weekday; label: string }[] = [
  { value: 'mon', label: weekdayLabel('mon') },
  { value: 'tue', label: weekdayLabel('tue') },
  { value: 'wed', label: weekdayLabel('wed') },
  { value: 'thu', label: weekdayLabel('thu') },
  { value: 'fri', label: weekdayLabel('fri') },
  { value: 'sat', label: weekdayLabel('sat') },
  { value: 'sun', label: weekdayLabel('sun') },
];
