import { describe, expect, it } from 'vitest';
import { nextOutcome } from './outcome-navigation';

describe('nextOutcome', () => {
  it('moves forward through outcomes', () => {
    expect(nextOutcome('done', 'next')).toBe('skipped');
    expect(nextOutcome('skipped', 'next')).toBe('notCompleted');
    expect(nextOutcome('notCompleted', 'next')).toBe('done');
  });

  it('moves backward through outcomes', () => {
    expect(nextOutcome('done', 'prev')).toBe('notCompleted');
    expect(nextOutcome('skipped', 'prev')).toBe('done');
    expect(nextOutcome('notCompleted', 'prev')).toBe('skipped');
  });

  it('treats null as done for navigation', () => {
    expect(nextOutcome(null, 'next')).toBe('skipped');
    expect(nextOutcome(null, 'prev')).toBe('notCompleted');
  });
});
