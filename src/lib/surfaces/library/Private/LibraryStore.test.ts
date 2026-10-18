import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LibraryStore } from './LibraryStore.svelte';
import * as api from './library-test-api';
import { emptyLibrary } from './library-test-api';

vi.mock('../../../api', () => import('./library-test-api'));

beforeEach(() => {
  api.resetLibraryApi();
  api.recurringTasks.mockResolvedValue([api.activeRecurring, api.archivedRecurring]);
});

describe('LibraryStore loading', () => {
  it("loads everything, archived included, with this and next week's focus", async () => {
    const store = new LibraryStore();
    await store.load();

    expect(api.library).toHaveBeenCalledWith(true);
    expect(store.view).toEqual(emptyLibrary);
    expect(api.weeklyFocus).toHaveBeenCalledWith('2026-W32');
    expect(api.weeklyFocus).toHaveBeenCalledWith('2026-W33');
    expect([...store.focusIds]).toEqual(['t-this']);
    expect([...store.nextFocusIds]).toEqual(['t-next']);
    expect(store.loading).toBe(false);
  });

  it('splits recurring rules into active and archived', async () => {
    const store = new LibraryStore();
    await store.load();

    expect(store.recurringTasks.map((rule) => rule.id)).toEqual(['r1']);
    expect(store.archivedRecurringTasks.map((rule) => rule.id)).toEqual(['r2']);
  });
});

describe('LibraryStore navigation', () => {
  it('clears the selection when the tab changes', async () => {
    const store = new LibraryStore();
    store.select({ kind: 'task', id: 't1' });
    store.setTab('habits');

    expect(store.tab).toBe('habits');
    expect(store.selected).toBeNull();
  });
});

describe('LibraryStore lifecycle mutations', () => {
  it('reloads after archiving an entity', async () => {
    const store = new LibraryStore();
    await store.load();
    api.library.mockClear();

    await store.setArchived({ kind: 'goal', id: 'g1' }, /* archived= */ true);
    expect(api.archiveEntity).toHaveBeenCalledWith({ kind: 'goal', id: 'g1' });
    expect(api.library).toHaveBeenCalledOnce();
  });

  it('archives and restores recurring rules', async () => {
    const store = new LibraryStore();
    await store.load();

    await store.setRecurringArchived('r1', /* archived= */ true);
    await store.setRecurringArchived('r2', /* archived= */ false);
    expect(api.archiveRecurringTask).toHaveBeenCalledWith('r1');
    expect(api.restoreRecurringTask).toHaveBeenCalledWith('r2');
  });

  it('creates a recurring task with daily recurrence', async () => {
    const store = new LibraryStore();
    await store.createRecurringTask('Daily standup', { kind: 'daily' });
    expect(api.createRecurringTask).toHaveBeenCalledWith('Daily standup', { kind: 'daily' });
  });
});

describe('LibraryStore task mutations', () => {
  it('creates a task with oneOff then reloads', async () => {
    const store = new LibraryStore();
    await store.createEntity({ kind: 'task', title: 'Pay rent', oneOff: false });
    expect(api.createTask).toHaveBeenCalledWith('Pay rent', false);
    expect(api.library).toHaveBeenCalled();
  });

  it('completes a task from its row; a one-off then archives itself server-side', async () => {
    const store = new LibraryStore();
    await store.load();

    await store.actions.toggleDone({ id: 't1', title: 'Send invoice', done: false, archived: false });
    expect(api.completeTask).toHaveBeenCalledWith('t1', undefined);
  });

  it("adds a task to next week's focus from the menu", async () => {
    const store = new LibraryStore();
    await store.load();
    const next = store.actions.focusTargets.find((target) => target.which === 'nextWeek')!;

    await store.actions.setFocus(next, 't1');
    expect(api.addToFocus).toHaveBeenCalledWith('2026-W33', 't1');
  });

  it('renames a habit then reloads', async () => {
    const store = new LibraryStore();
    await store.renameHabit('h1', 'Evening pages');
    expect(api.renameHabit).toHaveBeenCalledWith('h1', 'Evening pages');
  });
});
