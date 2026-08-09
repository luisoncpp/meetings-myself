import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import MarkdownEditorHarness from '../../../ui/Private/MarkdownEditor.harness.svelte';
import WeeklyReview from './WeeklyReview.svelte';

const openCurrentReview = vi.hoisted(() => vi.fn());
const openWeeklyReview = vi.hoisted(() => vi.fn());
const saveReflection = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const todayView = vi.hoisted(() => vi.fn());
const library = vi.hoisted(() => vi.fn());
const weeklyFocus = vi.hoisted(() => vi.fn());

vi.mock('../../../ui', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../../ui')>();
  return { ...original, MarkdownEditor: MarkdownEditorHarness };
});

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
afterEach(() => {
  vi.useRealTimers();
});

describe('WeeklyReview content', () => {
  it('shows the prior report, the regenerated summary, and next week Focus', async () => {
    render(WeeklyReview);
    expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('2026-W32');
    expect(screen.getByText(/Last week happened/)).toBeInTheDocument();
    expect(screen.getByText('Prepare portfolio')).toBeInTheDocument();
    const editor = screen.getByRole('textbox', { name: 'Reflection' }) as HTMLTextAreaElement;
    expect(editor.value).toContain('A quiet week');
  });

  it('shows habit counts without scoring them', async () => {
    render(WeeklyReview);
    await screen.findByRole('heading', { level: 1 });
    const habits = screen.getByRole('table', { name: /habits/i });
    expect(habits).toHaveTextContent('Writing');
    expect(habits.textContent).not.toMatch(/%|streak|score/i);
  });

  it('links to the report file so the user knows it is theirs to edit', async () => {
    render(WeeklyReview);
    expect(await screen.findByText(/2026-W32-weekly-report\.md/)).toBeInTheDocument();
  });
});

describe('WeeklyReview reflection', () => {
  it('autosaves the reflection and says so', async () => {
    vi.useFakeTimers();
    render(WeeklyReview);
    const editor = await screen.findByRole('textbox', { name: 'Reflection' });

    fireEvent.input(editor, {
      target: { value: '## Reflection\n\nA quiet week.\n More thoughts.' },
    });
    expect(screen.getByRole('status')).toHaveTextContent(/unsaved|saving/i);

    await vi.advanceTimersByTimeAsync(2000);
    expect(saveReflection).toHaveBeenCalledWith('2026-W32', expect.stringContaining('More thoughts.'));
    expect(screen.getByRole('status')).toHaveTextContent(/saved/i);
  });

  it('saves pending reflection changes before the window closes', async () => {
    const { unmount } = render(WeeklyReview);
    const editor = await screen.findByRole('textbox', { name: 'Reflection' });
    fireEvent.input(editor, {
      target: { value: '## Reflection\n\nA quiet week.\n Late edit.' },
    });
    unmount();
    expect(saveReflection).toHaveBeenCalled();
  });
});
