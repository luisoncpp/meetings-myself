import { afterEach, describe, expect, it, vi } from 'vitest';
import { formatPlanDate, parsePlanDate } from './plan-date';

describe('plan-date', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('parses YYYY-MM-DD as a local calendar day', () => {
    vi.stubEnv('TZ', 'America/Los_Angeles');
    const date = parsePlanDate('2026-08-07');
    expect(date.getFullYear()).toBe(2026);
    expect(date.getMonth()).toBe(7);
    expect(date.getDate()).toBe(7);
  });

  it('keeps the calendar day in negative-offset zones', () => {
    vi.stubEnv('TZ', 'America/Los_Angeles');
    const options = { dateStyle: 'long' as const, timeZone: 'America/Los_Angeles' };
    const local = new Intl.DateTimeFormat('en-US', options).format(parsePlanDate('2026-08-07'));
    const utcTrap = new Intl.DateTimeFormat('en-US', options).format(new Date('2026-08-07'));

    expect(local).toMatch(/August 7, 2026/);
    expect(utcTrap).toMatch(/August 6, 2026/);
  });

  it('formats with a long date style', () => {
    expect(formatPlanDate('2026-08-07', 'en-US')).toMatch(/2026/);
    expect(formatPlanDate('2026-08-07', 'en-US')).toMatch(/7|07/);
  });
});
