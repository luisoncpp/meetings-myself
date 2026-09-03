import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import DailyPlan from './DailyPlan.svelte';

const todayView = vi.hoisted(() => vi.fn());
const yesterdayView = vi.hoisted(() => vi.fn());
const taskPool = vi.hoisted(() => vi.fn());
const completeTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const recordCheckIn = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  todayView,
  yesterdayView,
  taskPool,
  selectIntoPlan: vi.fn(),
  removeFromPlan: vi.fn(),
  reorderPlan: vi.fn(),
  quickAddTask: vi.fn(),
  recordCheckIn,
  completeTask,
  reopenTask: vi.fn(),
}));

const emptyToday = {
  date: '2026-08-07',
  week: '2026-W32',
  tasks: [] as const,
  habits: [] as const,
};

const leftoverTask = {
  id: 'y1',
  title: 'Finish the draft',
  state: 'open' as const,
  importance: 'unclassified' as const,
  urgency: 'unclassified' as const,
  deadline: null,
  overdue: false,
  archived: false,
  position: 0,
};

const leftoverHabit = {
  id: 'yh1',
  title: 'Evening stretch',
  cadence: { kind: 'everyDay' as const },
  archived: false,
  unpinned: false,
  outcome: null,
};

const leftoverPlan = {
  date: '2026-08-06',
  week: '2026-W32',
  tasks: [leftoverTask],
  habits: [leftoverHabit],
};

beforeEach(() => {
  vi.clearAllMocks();
  todayView.mockResolvedValue(emptyToday);
  yesterdayView.mockResolvedValue(null);
  taskPool.mockResolvedValue({ focus: [], rest: [] });
});

async function expandYesterday(): Promise<HTMLDetailsElement> {
  const heading = await screen.findByRole('heading', { name: 'Yesterday' });
  const panel = heading.closest('details');
  const toggle = heading.closest('summary');
  if (!(panel instanceof HTMLDetailsElement) || !toggle) {
    throw new Error('Yesterday panel is missing');
  }
  expect(panel.open).toBe(false);
  await userEvent.click(toggle);
  expect(panel.open).toBe(true);
  return panel;
}

describe('DailyPlan yesterday catch-up rendering', () => {
  it('shows leftovers when yesterday already has a plan', async () => {
    yesterdayView.mockResolvedValue(leftoverPlan);
    render(DailyPlan);
    const panel = await expandYesterday();
    expect(panel).toHaveTextContent('Finish the draft');
    expect(screen.getByRole('radiogroup', { name: /Evening stretch/ })).toBeInTheDocument();
  });
});

describe('DailyPlan yesterday catch-up completion', () => {
  it('completes a leftover against yesterday', async () => {
    yesterdayView.mockResolvedValue({ ...leftoverPlan, habits: [] });
    render(DailyPlan);
    await expandYesterday();
    await userEvent.click(screen.getByRole('button', { name: /mark done: finish the draft/i }));
    expect(completeTask).toHaveBeenCalledWith('y1', '2026-08-06');
  });
});

describe('DailyPlan yesterday catch-up check-in', () => {
  it('records a habit check-in against yesterday', async () => {
    yesterdayView.mockResolvedValue({ ...leftoverPlan, tasks: [] });
    render(DailyPlan);
    await expandYesterday();
    const group = screen.getByRole('radiogroup', { name: /Evening stretch/ });
    await userEvent.click(within(group).getByRole('radio', { name: 'Done' }));
    expect(recordCheckIn).toHaveBeenCalledWith('yh1', '2026-08-06', 'done');
  });
});
