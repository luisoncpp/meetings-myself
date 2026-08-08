import { beforeEach, describe, expect, it, vi } from 'vitest';
import { WeeklyReviewStore } from './WeeklyReviewStore.svelte';

const openCurrentReview = vi.hoisted(() => vi.fn());
const openWeeklyReview = vi.hoisted(() => vi.fn());
const saveReflection = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const todayView = vi.hoisted(() => vi.fn());
const library = vi.hoisted(() => vi.fn());
const weeklyFocus = vi.hoisted(() => vi.fn());

vi.mock('../../../api', () => ({
  openCurrentReview,
  openWeeklyReview,
  saveReflection,
  todayView,
  library,
  weeklyFocus,
  createGoal: vi.fn().mockResolvedValue({ id: 'g1' }),
  achieveGoal: vi.fn().mockResolvedValue(undefined),
  link: vi.fn().mockResolvedValue({ id: 'a1' }),
  addToFocus: vi.fn().mockResolvedValue(undefined),
  removeFromFocus: vi.fn().mockResolvedValue(undefined),
}));

const sampleReview = {
  week: '2026-W32',
  summary: {
    week: '2026-W32',
    completed: ['Prepare portfolio'],
    stillOpen: 2,
    overdue: [] as string[],
    habits: [{ title: 'Writing', done: 4, skipped: 1, notCompleted: 2 }],
    goalsAchieved: [] as string[],
  },
  reflection: '## Reflection\n\nA quiet week.\n',
  previousReport: '## Week in review\n\nLast week happened.\n',
  nextWeekFocus: [] as const,
  reportPath: 'D:/Drive/weekly-reports/2026-W32-weekly-report.md',
};

function resetMocks(): void {
  vi.clearAllMocks();
  openCurrentReview.mockResolvedValue(sampleReview);
  openWeeklyReview.mockResolvedValue(sampleReview);
  todayView.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
  library.mockResolvedValue({ values: [], goals: [], habits: [], tasks: [] });
  weeklyFocus.mockResolvedValue({ id: 'f1', week: '2026-W33', tasks: [], createdAt: '2026-08-01' });
}

beforeEach(resetMocks);

describe('WeeklyReviewStore loading', () => {
  it('loads the current review and library context', async () => {
    const store = new WeeklyReviewStore();
    await store.load();

    expect(store.view?.week).toBe('2026-W32');
    expect(store.draftReflection).toContain('A quiet week');
    expect(store.library).not.toBeNull();
    expect(openCurrentReview).toHaveBeenCalled();
  });
});

describe('WeeklyReviewStore reflection', () => {
  it('autosaves after two seconds', async () => {
    vi.useFakeTimers();
    const store = new WeeklyReviewStore();
    await store.load();

    store.setReflection('## Reflection\n\nA quiet week.\n More thoughts.');
    expect(store.saveState).toBe('unsaved');

    await vi.advanceTimersByTimeAsync(2000);
    expect(saveReflection).toHaveBeenCalledWith(
      '2026-W32',
      expect.stringContaining('More thoughts.'),
    );
    expect(store.saveState).toBe('saved');
    vi.useRealTimers();
  });

  it('flushes pending edits on destroy', async () => {
    const store = new WeeklyReviewStore();
    await store.load();

    store.setReflection('## Reflection\n\nLate edit.');
    store.destroy();

    expect(saveReflection).toHaveBeenCalledWith('2026-W32', expect.stringContaining('Late edit.'));
  });
});

describe('WeeklyReviewStore navigation', () => {
  it('opens a specific week and marks it historical when not current', async () => {
    const store = new WeeklyReviewStore();
    await store.load();

    openWeeklyReview.mockResolvedValueOnce({ ...sampleReview, week: '2026-W31' });
    await store.openWeek('2026-W31');

    expect(store.view?.week).toBe('2026-W31');
    expect(store.isHistorical).toBe(true);
  });
});
