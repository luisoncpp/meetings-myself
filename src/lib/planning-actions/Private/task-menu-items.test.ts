import { describe, expect, it, vi } from 'vitest';

vi.mock('../../api', () => ({}));

import { TaskActions, type FocusTarget, type TaskRef } from './task-actions';
import { taskMenuItems } from './task-menu-items';

const open: TaskRef = { id: 't1', title: 'Send invoice', done: false, archived: false };

function actions(options: { focus?: FocusTarget[]; planDate?: string } = {}): TaskActions {
  return new TaskActions({
    run: async () => {},
    focusTargets: () => options.focus ?? [],
    planDate: options.planDate === undefined ? undefined : () => options.planDate!,
  });
}

function labels(task: TaskRef, host: TaskActions): string[] {
  return taskMenuItems(task, host, /* onrename= */ () => {}).map((item) => item.label);
}

describe('taskMenuItems', () => {
  it('offers done-and-archive, rename and archive for an open task', () => {
    expect(labels(open, actions())).toEqual(['Done and archive', 'Rename…', 'Archive']);
  });

  it('offers reopen and restore for a done, auto-archived task', () => {
    const done = { ...open, done: true, archived: true };
    expect(labels(done, actions())).toEqual(['Reopen', 'Rename…', 'Restore']);
  });

  it('names each focus by week and flips add/remove by membership', () => {
    const focus: FocusTarget[] = [
      { week: '2026-W32', which: 'thisWeek', taskIds: new Set(['t1']) },
      { week: '2026-W33', which: 'nextWeek', taskIds: new Set() },
    ];
    expect(labels(open, actions({ focus }))).toContain("Remove from this week's focus");
    expect(labels(open, actions({ focus }))).toContain("Add to next week's focus");
  });

  it('offers add or remove today only where a plan is shown', () => {
    const host = actions({ planDate: '2026-08-07' });
    expect(labels(open, host)).toContain('Add to today');
    expect(labels({ ...open, inToday: true }, host)).toContain('Remove from today');
    expect(labels(open, actions())).not.toContain('Add to today');
  });

  it('keeps the menu open for rename so the form can replace the items', () => {
    const rename = taskMenuItems(open, actions(), /* onrename= */ () => {}).find(
      (item) => item.id === 'rename',
    );
    expect(rename?.keepOpen).toBe(true);
  });
});
