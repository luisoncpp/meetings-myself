import { describe, expect, it } from 'vitest';
import { isRecurrenceValid, recurrenceLabel } from './recurrence-label';

describe('recurrenceLabel', () => {
  it('formats daily recurrence', () => {
    expect(recurrenceLabel({ kind: 'daily' })).toBe('Daily');
  });

  it('formats weekdays recurrence', () => {
    expect(recurrenceLabel({ kind: 'weekdays' })).toBe('Weekdays');
  });

  it('formats weekly recurrence with weekday', () => {
    expect(recurrenceLabel({ kind: 'weekly', weekday: 'thu' })).toBe('Weekly · Thursday');
  });

  it('formats monthly recurrence', () => {
    expect(recurrenceLabel({ kind: 'monthlyDay', day: 15 })).toBe('Monthly · day 15');
  });
});

describe('isRecurrenceValid', () => {
  it('accepts daily and weekdays without extra fields', () => {
    expect(isRecurrenceValid('daily', 'mon', '1')).toBe(true);
    expect(isRecurrenceValid('weekdays', 'mon', '1')).toBe(true);
  });

  it('requires a weekday for weekly recurrence', () => {
    expect(isRecurrenceValid('weekly', '' as 'mon', '1')).toBe(false);
    expect(isRecurrenceValid('weekly', 'fri', '1')).toBe(true);
  });

  it('rejects invalid monthly day values', () => {
    expect(isRecurrenceValid('monthlyDay', 'mon', '0')).toBe(false);
    expect(isRecurrenceValid('monthlyDay', 'mon', '32')).toBe(false);
    expect(isRecurrenceValid('monthlyDay', 'mon', '1.5')).toBe(false);
    expect(isRecurrenceValid('monthlyDay', 'mon', '15')).toBe(true);
  });
});
