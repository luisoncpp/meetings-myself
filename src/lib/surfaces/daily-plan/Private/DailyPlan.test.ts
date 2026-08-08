import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { formatPlanDate } from '../../../domain';
import DailyPlan from './DailyPlan.svelte';

const todayView = vi.hoisted(() => vi.fn());
const taskPool = vi.hoisted(() => vi.fn());
const selectIntoPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const removeFromPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const reorderPlan = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const quickAddTask = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 't9' }));
const recordCheckIn = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const completeTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const reopenTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  todayView,
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
      importance: 'unclassified' as const,
      urgency: 'unclassified' as const,
      deadline: null,
      overdue: false,
      archived: false,
    },
  ],
  rest: [
    {
      id: 'r1',
      title: 'Something else',
      state: 'open' as const,
      importance: 'unclassified' as const,
      urgency: 'unclassified' as const,
      deadline: null,
      overdue: false,
      archived: false,
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  todayView.mockResolvedValue(emptyPlan);
  taskPool.mockResolvedValue({ focus: [], rest: [] });
});

describe('DailyPlan rendering', () => {
  it('shows the date, the ordered tasks, and the habits due today', async () => {
    todayView.mockResolvedValue(loadedPlan);
    render(DailyPlan);
    expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent(
      formatPlanDate('2026-08-07'),
    );
    expect(screen.getByText('Draft the letter')).toBeInTheDocument();
    expect(screen.getByRole('radiogroup', { name: /Writing practice/ })).toBeInTheDocument();
  });

  it('shows archived and overdue entries in place with honest labels', async () => {
    todayView.mockResolvedValue(archivedFixture);
    render(DailyPlan);
    expect(await screen.findByText('Old idea')).toBeInTheDocument();
    expect(screen.getByText('Archived')).toBeInTheDocument();
    expect(screen.getByText('Overdue')).toBeInTheDocument();
  });
});

describe('DailyPlan actions', () => {
  it('still lets an archived entry be completed', async () => {
    todayView.mockResolvedValue(archivedFixture);
    render(DailyPlan);
    await userEvent.click(await screen.findByRole('checkbox', { name: /Old idea/ }));
    expect(completeTask).toHaveBeenCalledWith('t1');
  });

  it('quick-adds a task into today', async () => {
    render(DailyPlan);
    await userEvent.type(await screen.findByLabelText(/add a task/i), 'Call the bank{Enter}');
    expect(quickAddTask).toHaveBeenCalledWith('Call the bank');
  });

  it('lists Weekly Focus tasks before the rest of the pool', async () => {
    taskPool.mockResolvedValue(focusPool);
    render(DailyPlan);
    const pool = await screen.findByRole('region', { name: /task pool/i });
    const titles = within(pool).getAllByRole('button', { name: /add to today/i });
    expect(titles[0]).toHaveAccessibleName(/Prepare portfolio/);
  });
});
