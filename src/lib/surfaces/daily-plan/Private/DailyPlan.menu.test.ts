import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import DailyPlan from './DailyPlan.svelte';

const completeTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const archiveEntity = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const renameTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  todayView: vi.fn().mockResolvedValue({
    date: '2026-08-07',
    week: '2026-W32',
    tasks: [
      {
        id: 't1',
        title: 'Draft the letter',
        state: 'open',
        importance: 'unclassified',
        urgency: 'unclassified',
        deadline: null,
        overdue: false,
        archived: false,
        completed: false,
        position: 0,
      },
    ],
    habits: [],
  }),
  yesterdayView: vi.fn().mockResolvedValue({ date: '2026-08-06', week: '2026-W32', tasks: [], habits: [] }),
  taskPool: vi.fn().mockResolvedValue({ focus: [], rest: [] }),
  weeklyFocus: vi.fn().mockResolvedValue({ id: 'f', week: '2026-W32', tasks: [], createdAt: '' }),
  completeTask,
  archiveEntity,
  renameTask,
}));

beforeEach(() => vi.clearAllMocks());

describe('DailyPlan task menu', () => {
  it('marks a task done and archives it from the right-click menu', async () => {
    render(DailyPlan);
    await fireEvent.contextMenu(await screen.findByRole('group', { name: 'Draft the letter' }));
    await userEvent.click(screen.getByRole('menuitem', { name: 'Done and archive' }));
    expect(completeTask).toHaveBeenCalledWith('t1', '2026-08-07');
    expect(archiveEntity).toHaveBeenCalledWith({ kind: 'task', id: 't1' });
  });

  it('renames a task from the menu', async () => {
    render(DailyPlan);
    await userEvent.click(await screen.findByRole('button', { name: /more actions for draft the letter/i }));
    await userEvent.click(screen.getByRole('menuitem', { name: /rename/i }));
    const input = screen.getByRole('textbox', { name: 'New title' });
    await userEvent.clear(input);
    await userEvent.type(input, 'Draft the cover letter{Enter}');
    expect(renameTask).toHaveBeenCalledWith('t1', 'Draft the cover letter');
  });
});
