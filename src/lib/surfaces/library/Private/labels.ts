import type { Classification, HabitStrength, Weekday } from '../../../domain';
import { t } from '../../../i18n';

export function strengthLabel(value: HabitStrength): string {
  return t(`domain.habitStrength.${value}`);
}

export function classificationLabel(value: Classification): string {
  return t(`domain.classification.${value}`);
}

export function weekdayLabel(value: Weekday): string {
  return t(`domain.weekday.${value}`);
}

export const STRENGTH_VALUES: readonly HabitStrength[] = [
  'reminderDependent',
  'cueTriggered',
  'strengthening',
  'established',
];

export const CLASSIFICATION_VALUES: readonly Classification[] = ['unclassified', 'low', 'high'];

export const WEEKDAY_VALUES: readonly Weekday[] = [
  'mon',
  'tue',
  'wed',
  'thu',
  'fri',
  'sat',
  'sun',
];
