import { describe, expect, it, vi } from 'vitest';
import type { CheckInOutcome, Recurrence } from '../domain';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

describe('todayView', () => {
  it('calls today_view with no args', async () => {
    invoke.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
    const { todayView } = await import('./index');
    await todayView();
    expect(invoke).toHaveBeenCalledWith('today_view');
  });
});

describe('taskPool', () => {
  it('calls task_pool with no args', async () => {
    invoke.mockResolvedValue({ focus: [], rest: [] });
    const { taskPool } = await import('./index');
    await taskPool();
    expect(invoke).toHaveBeenCalledWith('task_pool');
  });
});

describe('selectIntoPlan', () => {
  it('forwards date and task', async () => {
    invoke.mockResolvedValue(undefined);
    const { selectIntoPlan } = await import('./index');
    await selectIntoPlan('2026-08-07', 't1');
    expect(invoke).toHaveBeenCalledWith('select_into_plan', { date: '2026-08-07', task: 't1' });
  });
});

describe('removeFromPlan', () => {
  it('forwards date and task', async () => {
    invoke.mockResolvedValue(undefined);
    const { removeFromPlan } = await import('./index');
    await removeFromPlan('2026-08-07', 't1');
    expect(invoke).toHaveBeenCalledWith('remove_from_plan', { date: '2026-08-07', task: 't1' });
  });
});

describe('reorderPlan', () => {
  it('forwards date and order', async () => {
    invoke.mockResolvedValue(undefined);
    const { reorderPlan } = await import('./index');
    await reorderPlan('2026-08-07', ['t2', 't1']);
    expect(invoke).toHaveBeenCalledWith('reorder_plan', {
      date: '2026-08-07',
      order: ['t2', 't1'],
    });
  });
});

describe('addHabitToPlan', () => {
  it('forwards date and habit', async () => {
    invoke.mockResolvedValue(undefined);
    const { addHabitToPlan } = await import('./index');
    await addHabitToPlan('2026-08-07', 'h1');
    expect(invoke).toHaveBeenCalledWith('add_habit_to_plan', { date: '2026-08-07', habit: 'h1' });
  });
});

describe('quickAddTask', () => {
  it('forwards the title', async () => {
    invoke.mockResolvedValue({});
    const { quickAddTask } = await import('./index');
    await quickAddTask('Call the bank');
    expect(invoke).toHaveBeenCalledWith('quick_add_task', { title: 'Call the bank' });
  });
});

describe('recordCheckIn', () => {
  it('forwards habit, date, and outcome', async () => {
    const outcome: CheckInOutcome = 'notCompleted';
    invoke.mockResolvedValue(undefined);
    const { recordCheckIn } = await import('./index');
    await recordCheckIn('h1', '2026-08-07', outcome);
    expect(invoke).toHaveBeenCalledWith('record_check_in', {
      habit: 'h1',
      date: '2026-08-07',
      outcome,
    });
  });
});

describe('weeklyFocus', () => {
  it('forwards the week label', async () => {
    invoke.mockResolvedValue({ id: '2026-W32', week: '2026-W32', tasks: [], createdAt: '' });
    const { weeklyFocus } = await import('./index');
    await weeklyFocus('2026-W32');
    expect(invoke).toHaveBeenCalledWith('weekly_focus', { week: '2026-W32' });
  });
});

describe('addToFocus', () => {
  it('forwards week and task', async () => {
    invoke.mockResolvedValue(undefined);
    const { addToFocus } = await import('./index');
    await addToFocus('2026-W32', 't1');
    expect(invoke).toHaveBeenCalledWith('add_to_focus', { week: '2026-W32', task: 't1' });
  });
});

describe('removeFromFocus', () => {
  it('forwards week and task', async () => {
    invoke.mockResolvedValue(undefined);
    const { removeFromFocus } = await import('./index');
    await removeFromFocus('2026-W32', 't1');
    expect(invoke).toHaveBeenCalledWith('remove_from_focus', { week: '2026-W32', task: 't1' });
  });
});

describe('createRecurringTask', () => {
  it('forwards title and recurrence', async () => {
    const recurrence: Recurrence = { kind: 'daily' };
    invoke.mockResolvedValue({});
    const { createRecurringTask } = await import('./index');
    await createRecurringTask('Pay rent', recurrence);
    expect(invoke).toHaveBeenCalledWith('create_recurring_task', {
      title: 'Pay rent',
      recurrence,
    });
  });
});

describe('recurringTasks', () => {
  it('calls recurring_tasks with no args', async () => {
    invoke.mockResolvedValue([]);
    const { recurringTasks } = await import('./index');
    await recurringTasks();
    expect(invoke).toHaveBeenCalledWith('recurring_tasks');
  });
});

describe('archiveRecurringTask', () => {
  it('forwards the rule id', async () => {
    invoke.mockResolvedValue(undefined);
    const { archiveRecurringTask } = await import('./index');
    await archiveRecurringTask('r1');
    expect(invoke).toHaveBeenCalledWith('archive_recurring_task', { rule: 'r1' });
  });
});

describe('restoreRecurringTask', () => {
  it('forwards the rule id', async () => {
    invoke.mockResolvedValue(undefined);
    const { restoreRecurringTask } = await import('./index');
    await restoreRecurringTask('r1');
    expect(invoke).toHaveBeenCalledWith('restore_recurring_task', { rule: 'r1' });
  });
});
