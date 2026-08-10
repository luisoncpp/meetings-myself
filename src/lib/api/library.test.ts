import { describe, expect, it, vi } from 'vitest';
import type { AssociationEnd, Cadence } from '../domain';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

describe('library', () => {
  it('forwards includeArchived to the library command', async () => {
    invoke.mockResolvedValue({ values: [], goals: [], habits: [], tasks: [] });
    const { library } = await import('./index');
    await library(/*includeArchived=*/true);
    expect(invoke).toHaveBeenCalledWith('library', { includeArchived: true });
  });
});

describe('createTask', () => {
  it('forwards the title and oneOff default', async () => {
    invoke.mockResolvedValue({});
    const { createTask } = await import('./index');
    await createTask('Draft the letter');
    expect(invoke).toHaveBeenCalledWith('create_task', {
      title: 'Draft the letter',
      oneOff: true,
    });
  });

  it('forwards oneOff when set to false', async () => {
    invoke.mockResolvedValue({});
    const { createTask } = await import('./index');
    await createTask('Pay rent', /*oneOff=*/false);
    expect(invoke).toHaveBeenCalledWith('create_task', { title: 'Pay rent', oneOff: false });
  });
});

describe('setTaskOneOff', () => {
  it('forwards task and oneOff', async () => {
    invoke.mockResolvedValue(undefined);
    const { setTaskOneOff } = await import('./index');
    await setTaskOneOff('t1', /*oneOff=*/false);
    expect(invoke).toHaveBeenCalledWith('set_task_one_off', { task: 't1', oneOff: false });
  });
});

describe('createValue', () => {
  it('forwards the title', async () => {
    invoke.mockResolvedValue({});
    const { createValue } = await import('./index');
    await createValue('Integrity');
    expect(invoke).toHaveBeenCalledWith('create_value', { title: 'Integrity' });
  });
});

describe('createGoal', () => {
  it('forwards title and targetDate', async () => {
    invoke.mockResolvedValue({});
    const { createGoal } = await import('./index');
    await createGoal('Career', '2026-12-01');
    expect(invoke).toHaveBeenCalledWith('create_goal', {
      title: 'Career',
      targetDate: '2026-12-01',
    });
  });
});

describe('createHabit', () => {
  it('forwards title and cadence', async () => {
    const cadence: Cadence = { kind: 'onWeekdays', days: ['mon', 'wed'] };
    invoke.mockResolvedValue({});
    const { createHabit } = await import('./index');
    await createHabit('Writing', cadence);
    expect(invoke).toHaveBeenCalledWith('create_habit', { title: 'Writing', cadence });
  });
});

describe('archiveEntity', () => {
  it('forwards the association end', async () => {
    const end: AssociationEnd = { kind: 'task', id: 't1' };
    invoke.mockResolvedValue(undefined);
    const { archiveEntity } = await import('./index');
    await archiveEntity(end);
    expect(invoke).toHaveBeenCalledWith('archive_entity', { end });
  });
});

describe('restoreEntity', () => {
  it('forwards the association end', async () => {
    const end: AssociationEnd = { kind: 'goal', id: 'g1' };
    invoke.mockResolvedValue(undefined);
    const { restoreEntity } = await import('./index');
    await restoreEntity(end);
    expect(invoke).toHaveBeenCalledWith('restore_entity', { end });
  });
});

describe('completeTask', () => {
  it('forwards the task id', async () => {
    invoke.mockResolvedValue(undefined);
    const { completeTask } = await import('./index');
    await completeTask('t1');
    expect(invoke).toHaveBeenCalledWith('complete_task', { task: 't1' });
  });
});

describe('reopenTask', () => {
  it('forwards the task id', async () => {
    invoke.mockResolvedValue(undefined);
    const { reopenTask } = await import('./index');
    await reopenTask('t1');
    expect(invoke).toHaveBeenCalledWith('reopen_task', { task: 't1' });
  });
});

describe('achieveGoal', () => {
  it('forwards the goal id', async () => {
    invoke.mockResolvedValue(undefined);
    const { achieveGoal } = await import('./index');
    await achieveGoal('g1');
    expect(invoke).toHaveBeenCalledWith('achieve_goal', { goal: 'g1' });
  });
});

describe('unachieveGoal', () => {
  it('forwards the goal id', async () => {
    invoke.mockResolvedValue(undefined);
    const { unachieveGoal } = await import('./index');
    await unachieveGoal('g1');
    expect(invoke).toHaveBeenCalledWith('unachieve_goal', { goal: 'g1' });
  });
});

describe('classifyTask', () => {
  it('forwards classification fields', async () => {
    invoke.mockResolvedValue(undefined);
    const { classifyTask } = await import('./index');
    await classifyTask('t1', 'high', 'low');
    expect(invoke).toHaveBeenCalledWith('classify_task', {
      task: 't1',
      importance: 'high',
      urgency: 'low',
    });
  });
});

describe('setTaskDeadline', () => {
  it('forwards task and deadline', async () => {
    invoke.mockResolvedValue(undefined);
    const { setTaskDeadline } = await import('./index');
    await setTaskDeadline('t1', '2026-08-06');
    expect(invoke).toHaveBeenCalledWith('set_task_deadline', {
      task: 't1',
      deadline: '2026-08-06',
    });
  });
});

describe('setHabitCadence', () => {
  it('forwards habit and cadence', async () => {
    const cadence: Cadence = { kind: 'everyDay' };
    invoke.mockResolvedValue(undefined);
    const { setHabitCadence } = await import('./index');
    await setHabitCadence('h1', cadence);
    expect(invoke).toHaveBeenCalledWith('set_habit_cadence', { habit: 'h1', cadence });
  });
});

describe('setHabitPinned', () => {
  it('forwards habit and pinned flag', async () => {
    invoke.mockResolvedValue(undefined);
    const { setHabitPinned } = await import('./index');
    await setHabitPinned('h1', /*pinned=*/false);
    expect(invoke).toHaveBeenCalledWith('set_habit_pinned', { habit: 'h1', pinned: false });
  });
});

describe('setHabitStrength', () => {
  it('forwards habit and strength', async () => {
    invoke.mockResolvedValue(undefined);
    const { setHabitStrength } = await import('./index');
    await setHabitStrength('h1', 'established');
    expect(invoke).toHaveBeenCalledWith('set_habit_strength', {
      habit: 'h1',
      strength: 'established',
    });
  });
});

describe('link', () => {
  it('forwards both ends', async () => {
    const left: AssociationEnd = { kind: 'task', id: 't1' };
    const right: AssociationEnd = { kind: 'goal', id: 'g1' };
    invoke.mockResolvedValue({});
    const { link } = await import('./index');
    await link(left, right);
    expect(invoke).toHaveBeenCalledWith('link', { left, right });
  });
});

describe('unlink', () => {
  it('forwards the association id', async () => {
    invoke.mockResolvedValue(undefined);
    const { unlink } = await import('./index');
    await unlink('a1');
    expect(invoke).toHaveBeenCalledWith('unlink', { association: 'a1' });
  });
});

describe('associationsFor', () => {
  it('forwards the end', async () => {
    const end: AssociationEnd = { kind: 'goal', id: 'g1' };
    invoke.mockResolvedValue([]);
    const { associationsFor } = await import('./index');
    await associationsFor(end);
    expect(invoke).toHaveBeenCalledWith('associations_for', { end });
  });
});
