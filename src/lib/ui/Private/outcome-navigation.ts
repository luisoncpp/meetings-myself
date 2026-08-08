import type { CheckInOutcome } from '../../domain';

export const OUTCOME_OPTIONS: ReadonlyArray<{
  value: CheckInOutcome;
  label: string;
}> = [
  { value: 'done', label: 'Done' },
  { value: 'skipped', label: 'Skipped' },
  { value: 'notCompleted', label: 'Not completed' },
];

const OUTCOME_ORDER: readonly CheckInOutcome[] = OUTCOME_OPTIONS.map(
  (option) => option.value,
);

export function nextOutcome(
  current: CheckInOutcome | null,
  direction: 'next' | 'prev',
): CheckInOutcome {
  const index = current === null ? 0 : OUTCOME_ORDER.indexOf(current);
  const delta = direction === 'next' ? 1 : -1;
  const nextIndex =
    (index + delta + OUTCOME_ORDER.length) % OUTCOME_ORDER.length;
  const outcome = OUTCOME_ORDER[nextIndex];
  if (outcome !== undefined) return outcome;
  return 'done';
}
