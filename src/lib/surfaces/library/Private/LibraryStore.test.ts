import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LibraryStore } from './LibraryStore.svelte';

const library = vi.hoisted(() => vi.fn());
const todayView = vi.hoisted(() => vi.fn());
const weeklyFocus = vi.hoisted(() => vi.fn());
const archiveEntity = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const setHabitStrength = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  library,
  todayView,
  weeklyFocus,
  archiveEntity,
  setHabitStrength,
  createValue: vi.fn().mockResolvedValue({ id: 'v1' }),
  createGoal: vi.fn().mockResolvedValue({ id: 'g1' }),
  createHabit: vi.fn().mockResolvedValue({ id: 'h1' }),
  createTask: vi.fn().mockResolvedValue({ id: 't1' }),
  restoreEntity: vi.fn().mockResolvedValue(undefined),
  achieveGoal: vi.fn().mockResolvedValue(undefined),
  unachieveGoal: vi.fn().mockResolvedValue(undefined),
  classifyTask: vi.fn().mockResolvedValue(undefined),
  setTaskDeadline: vi.fn().mockResolvedValue(undefined),
  completeTask: vi.fn().mockResolvedValue(undefined),
  reopenTask: vi.fn().mockResolvedValue(undefined),
  setHabitCadence: vi.fn().mockResolvedValue(undefined),
  setHabitPinned: vi.fn().mockResolvedValue(undefined),
  link: vi.fn().mockResolvedValue({ id: 'a1' }),
  unlink: vi.fn().mockResolvedValue(undefined),
  addToFocus: vi.fn().mockResolvedValue(undefined),
  removeFromFocus: vi.fn().mockResolvedValue(undefined),
}));

const emptyLibrary = {
  values: [],
  goals: [],
  habits: [],
  tasks: [],
};

function resetMocks(): void {
  vi.clearAllMocks();
  library.mockResolvedValue(emptyLibrary);
  todayView.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
  weeklyFocus.mockResolvedValue({ id: 'f1', week: '2026-W32', tasks: [], createdAt: '2026-08-01' });
}

beforeEach(resetMocks);

describe('LibraryStore loading', () => {
  it('loads the library and weekly focus together', async () => {
    const store = new LibraryStore();
    await store.load();

    expect(store.view).toEqual(emptyLibrary);
    expect(store.week).toBe('2026-W32');
    expect(store.focus?.tasks).toEqual([]);
    expect(store.loading).toBe(false);
  });
});

describe('LibraryStore archived toggle', () => {
  it('reloads with archived entries when requested', async () => {
    const store = new LibraryStore();
    await store.load();
    library.mockResolvedValueOnce({
      ...emptyLibrary,
      tasks: [
        {
          id: 't1',
          title: 'Old',
          state: 'archived',
          archived: true,
          importance: 'unclassified',
          urgency: 'unclassified',
          deadline: null,
          overdue: false,
        },
      ],
    });

    await store.setIncludeArchived(/* show= */ true);
    expect(library).toHaveBeenLastCalledWith(true);
    expect(store.view?.tasks).toHaveLength(1);
  });
});

describe('LibraryStore archive mutation', () => {
  it('reloads after archiving an entity', async () => {
    const store = new LibraryStore();
    await store.load();

    await store.archive({ kind: 'task', id: 't1' });
    expect(archiveEntity).toHaveBeenCalledWith({ kind: 'task', id: 't1' });
    expect(library).toHaveBeenCalledTimes(2);
  });
});

describe('LibraryStore error handling', () => {
  it('surfaces failures without clearing the loaded view', async () => {
    const store = new LibraryStore();
    await store.load();
    setHabitStrength.mockRejectedValueOnce(new Error('habit is archived'));

    await store.setHabitStrength('h1', 'established');
    expect(store.error).toMatch(/archived/);
    expect(store.view).toEqual(emptyLibrary);
  });
});
