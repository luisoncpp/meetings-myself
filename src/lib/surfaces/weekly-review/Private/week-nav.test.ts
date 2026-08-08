import { describe, expect, it } from 'vitest';
import { nextWeek, prevWeek } from './week-nav';

describe('week navigation', () => {
  it('steps to the previous ISO week', () => {
    expect(prevWeek('2026-W32')).toBe('2026-W31');
  });

  it('steps to the next ISO week', () => {
    expect(nextWeek('2026-W32')).toBe('2026-W33');
  });

  it('crosses a year boundary backward', () => {
    expect(prevWeek('2026-W01')).toBe('2025-W52');
  });

  it('crosses a year boundary forward', () => {
    expect(nextWeek('2025-W52')).toBe('2026-W01');
  });
});
