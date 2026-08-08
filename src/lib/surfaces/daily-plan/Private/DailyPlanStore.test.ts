import { beforeEach, describe, expect, it, vi } from 'vitest';
import { DailyPlanStore } from './DailyPlanStore.svelte';

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
      position,
    })),
    habits: [],
  };
}

function resetMocks(): void {
  vi.clearAllMocks();
  todayView.mockResolvedValue(planWith(['a', 'b', 'c']));
  taskPool.mockResolvedValue({ focus: [], rest: [] });
}

beforeEach(resetMocks);

describe('DailyPlanStore loading', () => {
  it('loads the plan and the pool together', async () => {
    const store = new DailyPlanStore();
    await store.load();

    expect(store.plan?.tasks).toHaveLength(3);
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

    await store.toggleCompletion(store.plan!.tasks[0]!);
    expect(completeTask).toHaveBeenCalledWith('a');

    todayView.mockResolvedValue({
      ...planWith(['a']),
      tasks: [{ ...planWith(['a']).tasks[0]!, state: 'completed' as const }],
    });
    await store.load();
    await store.toggleCompletion(store.plan!.tasks[0]!);
    expect(reopenTask).toHaveBeenCalledWith('a');
  });
});
