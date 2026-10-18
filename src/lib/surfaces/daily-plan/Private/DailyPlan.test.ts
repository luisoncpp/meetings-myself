import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { formatPlanDate } from '../../../domain';
import DailyPlan from './DailyPlan.svelte';

const todayView = vi.hoisted(() => vi.fn());
const yesterdayView = vi.hoisted(() => vi.fn());
const taskPool = vi.hoisted(() => vi.fn());
const selectIntoPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const removeFromPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const reorderPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const quickAddTask = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 't9' }));
const recordCheckIn = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const completeTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const reopenTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const weeklyFocus = vi.hoisted(() => vi.fn());

vi.mock('../../../api', () => ({
  weeklyFocus,
  todayView,
  yesterdayView,
  taskPool,
  selectIntoPlan,
  removeFromPlan,
  reorderPlan,
  quickAddTask,
  recordCheckIn,
  completeTask,
  reopenTask,
}));

const emptyPlan = {
  date: '2026-08-07',
  week: '2026-W32',
  tasks: [] as const,
  habits: [] as const,
};

const emptyYesterday = {
  date: '2026-08-06',
  week: '2026-W32',
  tasks: [] as const,
  habits: [] as const,
};

const archivedFixture = {
  date: '2026-08-07',
  week: '2026-W32',
  tasks: [
    {
      id: 't1',
      title: 'Old idea',
      state: 'archived' as const,
      importance: 'unclassified' as const,
      urgency: 'unclassified' as const,
      deadline: null,
      overdue: false,
      archived: true,
      completed: false,
      position: 0,
    },
    {
      id: 't2',
      title: 'File taxes',
      state: 'open' as const,
      importance: 'high' as const,
      urgency: 'high' as const,
      deadline: '2026-08-06',
      overdue: true,
      archived: false,
      completed: false,
      position: 1,
    },
  ],
  habits: [],
};

const loadedPlan = {
  date: '2026-08-07',
  week: '2026-W32',
  tasks: [
    {
      id: 't1',
      title: 'Draft the letter',
      state: 'open' as const,
      importance: 'high' as const,
      urgency: 'unclassified' as const,
      deadline: null,
      overdue: false,
      archived: false,
      completed: false,
      position: 0,
    },
  ],
  habits: [
    {
      id: 'h1',
      title: 'Writing practice',
      cadence: { kind: 'everyDay' as const },
      archived: false,
      unpinned: false,
      outcome: null,
    },
  ],
};

const focusPool = {
  focus: [
    {
      id: 'f1',
      title: 'Prepare portfolio',
      state: 'open' as const,
      importance: 'high' as const,
      urgency: 'high' as const,
      deadline: null,
      overdue: false,
      archived: false,
      oneOff: true,
      completedOn: null,
    },
  ],
  rest: [
    {
      id: 'r1',
      title: 'Something else',
      state: 'open' as const,
      importance: 'low' as const,
      urgency: 'low' as const,
      deadline: null,
      overdue: false,
      archived: false,
      oneOff: true,
      completedOn: null,
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  todayView.mockResolvedValue(emptyPlan);
  yesterdayView.mockResolvedValue(emptyYesterday);
  taskPool.mockResolvedValue({ focus: [], rest: [] });
  weeklyFocus.mockResolvedValue({ id: 'f', week: '2026-W32', tasks: [], createdAt: '' });
});

describe('DailyPlan rendering', () => {
  it('shows the date, the ordered tasks, and the habits due today', async () => {
    todayView.mockResolvedValue(loadedPlan);
    render(DailyPlan);
    expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent(
      formatPlanDate('2026-08-07', 'en-US'),
    );
    expect(screen.getByText('Draft the letter')).toBeInTheDocument();
    expect(screen.getByRole('radiogroup', { name: /Writing practice/ })).toBeInTheDocument();
  });

  it('shows a collapsed yesterday control below today', async () => {
    render(DailyPlan);
    const toggle = await screen.findByRole('button', { name: /yesterday/i });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
  });

  it('keeps archived entries in place and labels overdue ones', async () => {
    todayView.mockResolvedValue(archivedFixture);
    render(DailyPlan);
    expect(await screen.findByText('Old idea')).toBeInTheDocument();
    expect(screen.getByText('Overdue')).toBeInTheDocument();
  });

  it("marks plan tasks that are in this week's focus", async () => {
    todayView.mockResolvedValue(loadedPlan);
    weeklyFocus.mockResolvedValue({ id: 'f', week: '2026-W32', tasks: ['t1'], createdAt: '' });
    render(DailyPlan);
    const row = await screen.findByRole('group', { name: 'Draft the letter' });
    expect(within(row).getByText("In this week's focus")).toBeInTheDocument();
  });
});

describe('DailyPlan actions', () => {
  it('still lets an archived entry be completed', async () => {
    todayView.mockResolvedValue(archivedFixture);
    render(DailyPlan);
    await userEvent.click(await screen.findByRole('checkbox', { name: /mark done: old idea/i }));
    expect(completeTask).toHaveBeenCalledWith('t1', '2026-08-07');
  });

  it('quick-adds a task into today', async () => {
    render(DailyPlan);
    await userEvent.type(await screen.findByLabelText(/add a task/i), 'Call the bank{Enter}');
    expect(quickAddTask).toHaveBeenCalledWith('Call the bank');
  });

  it('lists Weekly Focus tasks before the rest of the pool', async () => {
    taskPool.mockResolvedValue(focusPool);
    render(DailyPlan);
    const pool = await screen.findByRole('list', { name: /task pool/i });
    const titles = within(pool).getAllByRole('button', { name: /add to today/i });
    expect(titles[0]).toHaveAccessibleName(/Prepare portfolio/);
  });

  it('shows only high importance and urgency in the pool', async () => {
    taskPool.mockResolvedValue(focusPool);
    render(DailyPlan);
    const pool = await screen.findByRole('list', { name: /task pool/i });
    expect(within(pool).getByText('Important')).toBeInTheDocument();
    expect(within(pool).getByText('Urgent')).toBeInTheDocument();
    expect(within(pool).queryByText(/low/i)).toBeNull();
  });
});
