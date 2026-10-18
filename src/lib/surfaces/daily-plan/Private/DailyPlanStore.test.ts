import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DailyPlanStore } from './DailyPlanStore.svelte';

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
const archiveEntity = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const addToFocus = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const removeFromFocus = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
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
  weeklyFocus,
  archiveEntity,
  addToFocus,
  removeFromFocus,
}));

function planWith(taskIds: string[]) {
  return {
    date: '2026-08-07',
    week: '2026-W32',
    tasks: taskIds.map((id, position) => ({
      id,
      title: id,
      state: 'open' as const,
      importance: 'unclassified' as const,
      urgency: 'unclassified' as const,
      deadline: null,
      overdue: false,
      archived: false,
      completed: false,
      position,
    })),
    habits: [],
  };
}

function resetMocks(): void {
  vi.clearAllMocks();
  todayView.mockResolvedValue(planWith(['a', 'b', 'c']));
  yesterdayView.mockResolvedValue({ ...planWith([]), date: '2026-08-06' });
  taskPool.mockResolvedValue({ focus: [], rest: [] });
  weeklyFocus.mockResolvedValue({ id: 'f', week: '2026-W32', tasks: ['b'], createdAt: '' });
}

beforeEach(resetMocks);

describe('DailyPlanStore loading', () => {
  it('loads the plan and the pool together', async () => {
    const store = new DailyPlanStore();
    await store.load();

    expect(store.plan?.tasks).toHaveLength(3);
    expect(store.yesterday?.date).toBe('2026-08-06');
    expect(store.pool).toEqual({ focus: [], rest: [] });
    expect(store.loading).toBe(false);
  });
});

describe('DailyPlanStore reorder', () => {
  it('applies a reorder optimistically and sends the new order', async () => {
    const store = new DailyPlanStore();
    await store.load();

    const pending = store.reorder(['c', 'a', 'b']);
    expect(store.plan?.tasks.map((task) => task.id)).toEqual(['c', 'a', 'b']);
    await pending;
    expect(reorderPlan).toHaveBeenCalledWith('2026-08-07', ['c', 'a', 'b']);
  });

  it('restores the server order when a reorder is rejected', async () => {
    const store = new DailyPlanStore();
    await store.load();
    reorderPlan.mockRejectedValueOnce(new Error('the proposed order is not a permutation'));

    await store.reorder(['c', 'a', 'b']);
    expect(store.plan?.tasks.map((task) => task.id)).toEqual(['a', 'b', 'c']);
    expect(store.error).toMatch(/permutation/);
  });
});

describe('DailyPlanStore mutations', () => {
  it('surfaces a failure without wiping the plan already on screen', async () => {
    const store = new DailyPlanStore();
    await store.load();
    selectIntoPlan.mockRejectedValueOnce(new Error('the task is archived'));

    await store.select('z');
    expect(store.error).toMatch(/archived/);
    expect(store.plan?.tasks).toHaveLength(3);
  });

  it('toggles completion in both directions', async () => {
    const store = new DailyPlanStore();
    await store.load();

    await store.toggleCompletion(store.plan!.tasks[0]!, '2026-08-07');
    expect(completeTask).toHaveBeenCalledWith('a', '2026-08-07');

    todayView.mockResolvedValue({
      ...planWith(['a']),
      // A done one-off archived itself; the plan still knows it is done.
      tasks: [{ ...planWith(['a']).tasks[0]!, state: 'archived' as const, archived: true, completed: true }],
    });
    await store.load();
    await store.toggleCompletion(store.plan!.tasks[0]!, '2026-08-07');
    expect(reopenTask).toHaveBeenCalledWith('a');
  });
});

describe('DailyPlanStore yesterday completion', () => {
  it('completes a leftover against yesterday', async () => {
    yesterdayView.mockResolvedValue({ ...planWith(['left']), date: '2026-08-06' });
    const store = new DailyPlanStore();
    await store.load();

    await store.toggleCompletion(store.yesterday!.tasks[0]!, '2026-08-06');
    expect(completeTask).toHaveBeenCalledWith('left', '2026-08-06');
  });
});

describe('DailyPlanStore yesterday check-in', () => {
  it('records a check-in against yesterday', async () => {
    yesterdayView.mockResolvedValue({
      date: '2026-08-06',
      week: '2026-W32',
      tasks: [],
      habits: [
        {
          id: 'h1',
          title: 'Writing',
          cadence: { kind: 'everyDay' as const },
          archived: false,
          unpinned: false,
          outcome: null,
        },
      ],
    });
    const store = new DailyPlanStore();
    await store.load();

    await store.checkIn('h1', 'done', '2026-08-06');
    expect(recordCheckIn).toHaveBeenCalledWith('h1', '2026-08-06', 'done');
  });
});

describe('DailyPlanStore task menu actions', () => {
  it("knows which plan tasks are in this week's focus", async () => {
    const store = new DailyPlanStore();
    await store.load();

    expect(weeklyFocus).toHaveBeenCalledWith('2026-W32');
    expect([...store.focusIds]).toEqual(['b']);
  });

  it('marks a task done and archives it, stamped with the plan date', async () => {
    const store = new DailyPlanStore();
    await store.load();

    await store.actions.finish({ id: 'a', title: 'a', done: false, archived: false });
    expect(completeTask).toHaveBeenCalledWith('a', '2026-08-07');
    expect(archiveEntity).toHaveBeenCalledWith({ kind: 'task', id: 'a' });
  });

  it("toggles this week's focus from the menu", async () => {
    const store = new DailyPlanStore();
    await store.load();
    const [target] = store.actions.focusTargets;

    await store.actions.setFocus(target!, 'a');
    await store.actions.setFocus(target!, 'b');
    expect(addToFocus).toHaveBeenCalledWith('2026-W32', 'a');
    expect(removeFromFocus).toHaveBeenCalledWith('2026-W32', 'b');
  });
});
