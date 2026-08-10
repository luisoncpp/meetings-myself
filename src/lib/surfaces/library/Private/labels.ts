import type { Classification, HabitStrength, Weekday } from '../../../domain';
import { t } from '../../../i18n';

function strengthLabel(value: HabitStrength): string {
  return t(`domain.habitStrength.${value}`);
}

function classificationLabel(value: Classification): string {
  return t(`domain.classification.${value}`);
}

export function weekdayLabel(value: Weekday): string {
  return t(`domain.weekday.${value}`);
}

export const STRENGTH_OPTIONS: { value: HabitStrength; label: string }[] = [
  { value: 'reminderDependent', label: strengthLabel('reminderDependent') },
  { value: 'cueTriggered', label: strengthLabel('cueTriggered') },
  { value: 'strengthening', label: strengthLabel('strengthening') },
  { value: 'established', label: strengthLabel('established') },
];

export const CLASSIFICATION_OPTIONS: { value: Classification; label: string }[] = [
  { value: 'unclassified', label: classificationLabel('unclassified') },
  { value: 'low', label: classificationLabel('low') },
  { value: 'high', label: classificationLabel('high') },
];

export const WEEKDAYS: { value: Weekday; label: string }[] = [
  { value: 'mon', label: weekdayLabel('mon') },
  { value: 'tue', label: weekdayLabel('tue') },
  { value: 'wed', label: weekdayLabel('wed') },
  { value: 'thu', label: weekdayLabel('thu') },
  { value: 'fri', label: weekdayLabel('fri') },
  { value: 'sat', label: weekdayLabel('sat') },
  { value: 'sun', label: weekdayLabel('sun') },
];
