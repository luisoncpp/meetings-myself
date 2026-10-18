// fallow-ignore-file unused-export
// fallow-ignore-file duplicate-export
import { vi } from 'vitest';

/** Shared `../../../api` mock for Library tests: `vi.mock(path, () => import('./library-test-api'))`. */
const resolved = (value?: unknown) => vi.fn().mockResolvedValue(value);

export const library = vi.fn();
export const todayView = vi.fn();
export const weeklyFocus = vi.fn();
export const recurringTasks = vi.fn();
export const setHabitStrength = resolved();
export const setHabitPinned = resolved();
export const achieveGoal = resolved();
export const createRecurringTask = resolved({ id: 'r-new' });
export const archiveRecurringTask = resolved();
export const restoreRecurringTask = resolved();
export const renameRecurringTask = resolved();
export const renameHabit = resolved();
export const createTask = resolved({ id: 't1' });
export const setTaskOneOff = resolved();
export const completeTask = resolved();
export const archiveEntity = resolved();
export const restoreEntity = resolved();
export const addToFocus = resolved();
export const removeFromFocus = resolved();
export const createValue = resolved({ id: 'v1' });
export const createGoal = resolved({ id: 'g1' });
export const createHabit = resolved({ id: 'h9' });
export const unachieveGoal = resolved();
export const classifyTask = resolved();
export const setTaskDeadline = resolved();
export const reopenTask = resolved();
export const renameTask = resolved();
export const setHabitCadence = resolved();
export const link = resolved({ id: 'a1' });
export const unlink = resolved();
export const uiLanguage = resolved('en');
export const setUiLanguage = resolved();

export const emptyLibrary = { values: [], goals: [], habits: [], tasks: [], associations: [] };

export const activeRecurring = {
  id: 'r1',
  title: 'Daily standup',
  recurrence: { kind: 'daily' as const },
  lifecycle: 'active' as const,
  startsOn: '2026-08-01',
  materializedThrough: null,
  createdAt: '2026-08-01',
};

export const archivedRecurring = {
  ...activeRecurring,
  id: 'r2',
  title: 'Old rule',
  lifecycle: 'archived' as const,
};

export const writingHabit = {
  id: 'h1',
  title: 'Writing',
  cadence: { kind: 'everyDay' as const },
  strength: 'reminderDependent' as const,
  pinned: true,
  archived: false,
};

export function task(id: string, title: string, extra: Record<string, unknown> = {}) {
  return {
    id,
    title,
    state: 'open',
    importance: 'unclassified',
    urgency: 'unclassified',
    deadline: null,
    overdue: false,
    archived: false,
    oneOff: true,
    completedOn: null,
    ...extra,
  };
}

export function withEntities(entities: Record<string, unknown[]>): void {
  library.mockResolvedValue({ ...emptyLibrary, ...entities });
}

/** Default state: empty library, week W32 (focus `t-this`), next week W33 (focus `t-next`). */
export function resetLibraryApi(): void {
  vi.clearAllMocks();
  library.mockResolvedValue(emptyLibrary);
  todayView.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
  weeklyFocus.mockImplementation(async (week: string) => ({
    id: `f-${week}`,
    week,
    tasks: week === '2026-W32' ? ['t-this'] : ['t-next'],
    createdAt: '2026-08-01',
  }));
  recurringTasks.mockResolvedValue([]);
}
