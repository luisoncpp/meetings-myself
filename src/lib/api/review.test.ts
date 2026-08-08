import { describe, expect, it, vi } from 'vitest';
import type { WeeklyReviewView, WeeklySummary } from '../domain';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

const sampleSummary: WeeklySummary = {
  week: '2026-W32',
  completed: ['Prepare portfolio'],
  stillOpen: 1,
  overdue: [],
  habits: [{ title: 'Writing', done: 2, skipped: 0, notCompleted: 1 }],
  goalsAchieved: [],
};

const sampleReview: WeeklyReviewView = {
  week: '2026-W32',
  summary: sampleSummary,
  reflection: '## Reflection\n\nA quiet week.\n',
  previousReport: null,
  nextWeekFocus: [],
  reportPath: '/sync/weekly-reports/2026-W32-weekly-report.md',
};

describe('openWeeklyReview', () => {
  it('forwards the week label', async () => {
    invoke.mockResolvedValue(sampleReview);
    const { openWeeklyReview } = await import('./index');
    await openWeeklyReview('2026-W32');
    expect(invoke).toHaveBeenCalledWith('open_weekly_review', { week: '2026-W32' });
  });
});

describe('openCurrentReview', () => {
  it('calls open_current_review with no args', async () => {
    invoke.mockResolvedValue(sampleReview);
    const { openCurrentReview } = await import('./index');
    await openCurrentReview();
    expect(invoke).toHaveBeenCalledWith('open_current_review');
  });
});

describe('saveReflection', () => {
  it('forwards week and reflection', async () => {
    invoke.mockResolvedValue(undefined);
    const { saveReflection } = await import('./index');
    await saveReflection('2026-W32', '## Reflection\n\nNotes.\n');
    expect(invoke).toHaveBeenCalledWith('save_reflection', {
      week: '2026-W32',
      reflection: '## Reflection\n\nNotes.\n',
    });
  });
});

describe('weeklySummary', () => {
  it('forwards the week label', async () => {
    invoke.mockResolvedValue(sampleSummary);
    const { weeklySummary } = await import('./index');
    await weeklySummary('2026-W32');
    expect(invoke).toHaveBeenCalledWith('weekly_summary', { week: '2026-W32' });
  });
});

describe('reportPath', () => {
  it('forwards the week label', async () => {
    invoke.mockResolvedValue('/sync/weekly-reports/2026-W32-weekly-report.md');
    const { reportPath } = await import('./index');
    const path = await reportPath('2026-W32');
    expect(invoke).toHaveBeenCalledWith('report_path', { week: '2026-W32' });
    expect(path).toBe('/sync/weekly-reports/2026-W32-weekly-report.md');
  });
});
