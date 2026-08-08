import type { Classification, HabitStrength, Weekday } from '../../../domain';

export const STRENGTH_OPTIONS: { value: HabitStrength; label: string }[] = [
  { value: 'reminderDependent', label: 'Reminder-dependent' },
  { value: 'cueTriggered', label: 'Cue-triggered' },
  { value: 'strengthening', label: 'Strengthening' },
  { value: 'established', label: 'Established' },
];

export const CLASSIFICATION_OPTIONS: { value: Classification; label: string }[] = [
  { value: 'unclassified', label: 'Unclassified' },
  { value: 'low', label: 'Low' },
  { value: 'high', label: 'High' },
];

export const WEEKDAYS: { value: Weekday; label: string }[] = [
  { value: 'mon', label: 'Monday' },
  { value: 'tue', label: 'Tuesday' },
  { value: 'wed', label: 'Wednesday' },
  { value: 'thu', label: 'Thursday' },
  { value: 'fri', label: 'Friday' },
  { value: 'sat', label: 'Saturday' },
  { value: 'sun', label: 'Sunday' },
];
