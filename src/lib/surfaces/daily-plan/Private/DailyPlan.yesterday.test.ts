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
  yesterdayView.mockResolvedValue(leftoverPlan);
  taskPool.mockResolvedValue({ focus: [], rest: [] });
});

async function expandYesterday(): Promise<void> {
  const toggle = await screen.findByRole('button', { name: /yesterday/i });
  expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await userEvent.click(toggle);
  expect(toggle).toHaveAttribute('aria-expanded', 'true');
}

describe('DailyPlan yesterday catch-up rendering', () => {
  it('shows a collapsed yesterday control when that day already has a plan', async () => {
    yesterdayView.mockResolvedValue(leftoverPlan);
    render(DailyPlan);
    const toggle = await screen.findByRole('button', { name: /yesterday/i });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByText('Finish the draft')).not.toBeInTheDocument();
  });

  it('reveals leftovers after the panel is opened', async () => {
    yesterdayView.mockResolvedValue(leftoverPlan);
    render(DailyPlan);
    await expandYesterday();
    expect(screen.getByText('Finish the draft')).toBeInTheDocument();
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
