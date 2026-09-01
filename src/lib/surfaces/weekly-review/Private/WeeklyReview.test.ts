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

const createGoal = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 'g1' }));
const createHabit = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 'h1' }));
const createTask = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 't1' }));

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
  createGoal,
  createHabit,
  createTask,
  achieveGoal: vi.fn().mockResolvedValue(undefined),
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

describe('WeeklyReview action bar', () => {
  it('creates a task from the action bar', async () => {
    render(WeeklyReview);
    await screen.findByRole('heading', { level: 1 });

    const newTaskBtn = screen.getByRole('button', { name: /new task/i });
    await fireEvent.click(newTaskBtn);

    const titleInput = screen.getByRole('textbox', { name: /task name/i });
    await fireEvent.input(titleInput, { target: { value: 'Review quarterly goals' } });

    const submitBtn = screen.getByRole('button', { name: /create task/i });
    await fireEvent.click(submitBtn);

    expect(createTask).toHaveBeenCalledWith('Review quarterly goals', true);
  });

  it('creates a habit from the action bar', async () => {
    render(WeeklyReview);
    await screen.findByRole('heading', { level: 1 });

    const newHabitBtn = screen.getByRole('button', { name: /new habit/i });
    await fireEvent.click(newHabitBtn);

    const titleInput = screen.getByRole('textbox', { name: /habit name/i });
    await fireEvent.input(titleInput, { target: { value: 'Meditation' } });

    const dayCheckbox = screen.getByRole('checkbox', { name: /monday/i });
    await fireEvent.click(dayCheckbox);

    const submitBtn = screen.getByRole('button', { name: /create habit/i });
    await fireEvent.click(submitBtn);

    expect(createHabit).toHaveBeenCalledWith('Meditation', {
      kind: 'onWeekdays',
      days: ['mon'],
    });
  });
});


