import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { localeStore } from '../../../i18n';
import Library from './Library.svelte';

const library = vi.hoisted(() => vi.fn());
const todayView = vi.hoisted(() => vi.fn());
const weeklyFocus = vi.hoisted(() => vi.fn());
const recurringTasks = vi.hoisted(() => vi.fn());
const setHabitStrength = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const associationsFor = vi.hoisted(() => vi.fn().mockResolvedValue([]));
const achieveGoal = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const createRecurringTask = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 'r-new' }));
const archiveRecurringTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const restoreRecurringTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const renameRecurringTask = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const createTask = vi.hoisted(() => vi.fn().mockResolvedValue({ id: 't1' }));
const setTaskOneOff = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  library,
  todayView,
  weeklyFocus,
  recurringTasks,
  setHabitStrength,
  associationsFor,
  createRecurringTask,
  archiveRecurringTask,
  restoreRecurringTask,
  renameRecurringTask,
  createValue: vi.fn().mockResolvedValue({ id: 'v1' }),
  createGoal: vi.fn().mockResolvedValue({ id: 'g1' }),
  createTask,
  setTaskOneOff,
  createHabit: vi.fn().mockResolvedValue({ id: 'h9' }),
  archiveEntity: vi.fn().mockResolvedValue(undefined),
  restoreEntity: vi.fn().mockResolvedValue(undefined),
  achieveGoal,
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
  uiLanguage: vi.fn().mockResolvedValue('en'),
  setUiLanguage: vi.fn().mockResolvedValue(undefined),
}));

const emptyLibrary = {
  values: [],
  goals: [],
  habits: [],
  tasks: [],
  associations: [],
};

const activeRecurring = {
  id: 'r1',
  title: 'Daily standup',
  recurrence: { kind: 'daily' as const },
  lifecycle: 'active' as const,
  startsOn: '2026-08-01',
  materializedThrough: null,
  createdAt: '2026-08-01',
};

const archivedRecurring = {
  id: 'r2',
  title: 'Old rule',
  recurrence: { kind: 'weekdays' as const },
  lifecycle: 'archived' as const,
  startsOn: '2026-08-01',
  materializedThrough: '2026-08-07',
  createdAt: '2026-08-01',
};

function resetApiMocks(): void {
  vi.clearAllMocks();
  void localeStore.setLocale('en', /*persist=*/ false);
  library.mockResolvedValue(emptyLibrary);
  todayView.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
  weeklyFocus.mockResolvedValue({ id: 'f1', week: '2026-W32', tasks: [], createdAt: '2026-08-01' });
  recurringTasks.mockResolvedValue([]);
}

beforeEach(resetApiMocks);

describe('Library weekly review actions', () => {
  it('offers every action the Weekly Review offers', async () => {
    render(Library);
    await screen.findByRole('heading', { name: 'Library' });

    expect(screen.getByRole('button', { name: /new goal/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /mark achieved/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /link/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /weekly focus/i })).toBeInTheDocument();
  });

  it('opens a goal picker when Mark achieved is clicked', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [{ id: 'g1', title: 'Ship it', achieved: false, archived: false, targetDate: null }],
      habits: [],
      tasks: [],
    });

    render(Library);
    await screen.findByText('Ship it');
    const toolbar = screen.getByRole('toolbar', { name: /quick actions/i });
    const markAchieved = within(toolbar).getByRole('button', { name: /mark achieved/i });
    await userEvent.click(markAchieved);
    expect(screen.getByLabelText(/goal to mark achieved/i)).toBeInTheDocument();
    expect(markAchieved).toHaveAttribute('aria-pressed', 'true');
    expect(within(toolbar).getByRole('button', { name: /new goal/i })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  it('keeps only one action panel open at a time', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [{ id: 'g1', title: 'Ship it', achieved: false, archived: false, targetDate: null }],
      habits: [],
      tasks: [],
    });

    render(Library);
    await screen.findByText('Ship it');
    const toolbar = screen.getByRole('toolbar', { name: /quick actions/i });

    await userEvent.click(within(toolbar).getByRole('button', { name: /mark achieved/i }));
    expect(screen.getByLabelText(/goal to mark achieved/i)).toBeInTheDocument();

    await userEvent.click(within(toolbar).getByRole('button', { name: /^link$/i }));
    expect(screen.queryByLabelText(/goal to mark achieved/i)).not.toBeInTheDocument();
    expect(screen.getByLabelText(/association editor/i)).toBeInTheDocument();
  });

  it('closes the active panel when its button is clicked again', async () => {
    render(Library);
    await screen.findByRole('heading', { name: 'Library' });
    const toolbar = screen.getByRole('toolbar', { name: /quick actions/i });

    const newGoal = within(toolbar).getByRole('button', { name: /new goal/i });
    await userEvent.click(newGoal);
    expect(screen.getByLabelText(/goal name/i)).toBeInTheDocument();

    await userEvent.click(newGoal);
    expect(screen.queryByLabelText(/goal name/i)).not.toBeInTheDocument();
  });

  it('achieves the selected goal from the picker', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [{ id: 'g1', title: 'Ship it', achieved: false, archived: false, targetDate: null }],
      habits: [],
      tasks: [],
    });

    render(Library);
    await screen.findByText('Ship it');
    const toolbar = screen.getByRole('toolbar', { name: /quick actions/i });
    await userEvent.click(within(toolbar).getByRole('button', { name: /mark achieved/i }));
    await userEvent.selectOptions(screen.getByLabelText(/goal to mark achieved/i), 'g1');
    const confirmButtons = screen.getAllByRole('button', { name: /mark achieved/i });
    const confirm = confirmButtons.at(-1);
    expect(confirm).toBeDefined();
    await userEvent.click(confirm!);
    expect(achieveGoal).toHaveBeenCalledWith('g1');
  });
});

function mockArchivedTaskLibrary(): void {
  library.mockImplementation((includeArchived: boolean) =>
    Promise.resolve({
      values: [],
      goals: [],
      habits: [],
      tasks: includeArchived
        ? [
            {
              id: 't1',
              title: 'Old idea',
              state: 'archived',
              archived: true,
              importance: 'unclassified',
              urgency: 'unclassified',
              deadline: null,
              overdue: false,
              oneOff: true,
            },
          ]
        : [],
    }),
  );
}

describe('Library archived entries', () => {
  it('hides archived entries until asked, then shows them labelled', async () => {
    mockArchivedTaskLibrary();

    render(Library);
    expect(await screen.findByText(/no tasks/i)).toBeInTheDocument();

    await userEvent.click(screen.getByRole('switch', { name: /show archived/i }));
    expect(await screen.findByText('Old idea')).toBeInTheDocument();
    expect(screen.getByText('Archived')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /restore/i })).toBeInTheDocument();
  });
});

describe('Library delete guard', () => {
  it('never offers to delete anything', async () => {
    render(Library);
    await screen.findByRole('heading', { name: 'Library' });
    expect(screen.queryByRole('button', { name: /delete/i })).not.toBeInTheDocument();
    expect(screen.queryByText(/permanently/i)).not.toBeInTheDocument();
  });
});

describe('Library habit strength', () => {
  it('lets Habit Strength be set by hand, without scoring it', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [],
      tasks: [],
      habits: [
        {
          id: 'h1',
          title: 'Writing',
          cadence: { kind: 'everyDay' },
          strength: 'reminderDependent',
          pinned: true,
          archived: false,
        },
      ],
    });

    render(Library);
    const strength = await screen.findByLabelText(/habit strength/i);
    expect(strength).toHaveValue('reminderDependent');

    await userEvent.selectOptions(strength, 'strengthening');
    expect(setHabitStrength).toHaveBeenCalledWith('h1', 'strengthening');

    expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
    expect(screen.getByRole('region', { name: /habits/i }).textContent).not.toMatch(
      /level|\d+\s*\/\s*4/i,
    );
  });

  it('shows habit strength and weekday labels in Spanish', async () => {
    await localeStore.setLocale('es', /*persist=*/ false);
    library.mockResolvedValue({
      values: [],
      goals: [],
      tasks: [],
      habits: [
        {
          id: 'h1',
          title: 'Writing',
          cadence: { kind: 'everyDay' },
          strength: 'reminderDependent',
          pinned: true,
          archived: false,
        },
      ],
    });

    render(Library);
    const strength = await screen.findByLabelText(/fortaleza del hábito/i);
    expect(strength).toHaveTextContent('Dependiente del recordatorio');
    expect(screen.getByRole('checkbox', { name: 'Lunes' })).toBeInTheDocument();
  });
});

describe('Library task fields', () => {
  it('exposes every editable field the domain offers for a task', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [],
      habits: [],
      tasks: [
        {
          id: 't1',
          title: 'File taxes',
          state: 'open',
          importance: 'unclassified',
          urgency: 'unclassified',
          deadline: null,
          overdue: false,
          archived: false,
          oneOff: true,
        },
      ],
    });

    render(Library);
    await screen.findByText('File taxes');
    for (const label of [/importance/i, /urgency/i, /deadline/i]) {
      expect(screen.getByLabelText(label)).toBeInTheDocument();
    }
    expect(screen.getByRole('checkbox', { name: /one-off/i })).toBeChecked();
    expect(screen.getByRole('button', { name: /mark done: file taxes/i })).toBeInTheDocument();
  });

  it('hides completion controls for non-one-off tasks', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [],
      habits: [],
      tasks: [
        {
          id: 't1',
          title: 'Pay rent',
          state: 'completed',
          importance: 'unclassified',
          urgency: 'unclassified',
          deadline: null,
          overdue: false,
          archived: false,
          oneOff: false,
        },
      ],
    });

    render(Library);
    await screen.findByText('Pay rent');
    expect(screen.queryByRole('button', { name: /mark done/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /reopen/i })).not.toBeInTheDocument();
  });

  it('shows done state for completed one-off tasks', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [],
      habits: [],
      tasks: [
        {
          id: 't1',
          title: 'Buy milk',
          state: 'completed',
          importance: 'unclassified',
          urgency: 'unclassified',
          deadline: null,
          overdue: false,
          archived: false,
          oneOff: true,
        },
      ],
    });

    render(Library);
    await screen.findByText('Buy milk');
    expect(screen.getByRole('button', { name: /reopen buy milk/i })).toHaveTextContent('Done');
  });

  it('creates a task with one-off unchecked when requested', async () => {
    render(Library);
    await userEvent.click(await screen.findByRole('button', { name: /add task/i }));
    await userEvent.type(screen.getByLabelText(/task name/i), 'Pay rent');
    await userEvent.click(screen.getByRole('checkbox', { name: /one-off/i }));
    await userEvent.click(screen.getByRole('button', { name: /create task/i }));
    expect(createTask).toHaveBeenCalledWith('Pay rent', /*oneOff=*/false);
  });

  it('toggles one-off on an existing task', async () => {
    library.mockResolvedValue({
      values: [],
      goals: [],
      habits: [],
      tasks: [
        {
          id: 't1',
          title: 'Pay rent',
          state: 'open',
          importance: 'unclassified',
          urgency: 'unclassified',
          deadline: null,
          overdue: false,
          archived: false,
          oneOff: true,
        },
      ],
    });

    render(Library);
    await screen.findByText('Pay rent');
    await userEvent.click(screen.getByRole('checkbox', { name: /one-off/i }));
    expect(setTaskOneOff).toHaveBeenCalledWith('t1', /*oneOff=*/false);
  });
});

describe('Library habit creation', () => {
  it('requires a cadence before a habit can be created', async () => {
    render(Library);
    await userEvent.click(await screen.findByRole('button', { name: /new habit/i }));
    await userEvent.type(screen.getByLabelText(/habit name/i), 'Meditation');

    expect(screen.getByRole('button', { name: /create habit/i })).toBeDisabled();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Monday' }));
    expect(screen.getByRole('button', { name: /create habit/i })).toBeEnabled();
  });
});

function recurringSection(): HTMLElement {
  return screen.getByLabelText(/recurring tasks/i);
}

describe('Library recurring tasks', () => {
  it('shows the section heading and empty state', async () => {
    render(Library);
    expect(await screen.findByRole('heading', { name: 'Recurring tasks' })).toBeInTheDocument();
    expect(screen.getByText('No recurring tasks yet.')).toBeInTheDocument();
  });

  it('creates a recurring task with daily recurrence', async () => {
    const created = {
      id: 'r-new',
      title: 'Water plants',
      recurrence: { kind: 'daily' as const },
      lifecycle: 'active' as const,
      startsOn: '2026-08-01',
      materializedThrough: null,
      createdAt: '2026-08-01',
    };
    recurringTasks.mockResolvedValueOnce([]).mockResolvedValue([created]);

    render(Library);
    await screen.findByRole('heading', { name: 'Recurring tasks' });

    const section = recurringSection();
    await userEvent.click(within(section).getByRole('button', { name: /new recurring task/i }));
    await userEvent.type(screen.getByLabelText(/recurring task title/i), 'Water plants');
    await userEvent.click(within(section).getByRole('button', { name: /^create$/i }));

    expect(createRecurringTask).toHaveBeenCalledWith('Water plants', { kind: 'daily' });
    expect(await screen.findByDisplayValue('Water plants')).toBeInTheDocument();
  });

  it('hides archived recurring tasks until show archived is toggled', async () => {
    recurringTasks.mockResolvedValue([activeRecurring, archivedRecurring]);

    render(Library);
    await screen.findByDisplayValue('Daily standup');
    expect(screen.queryByText('Old rule')).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole('switch', { name: /show archived/i }));

    const section = recurringSection();
    expect(await within(section).findByText('Old rule')).toBeInTheDocument();
    expect(within(section).getByText('Archived')).toBeInTheDocument();
    expect(within(section).getByRole('button', { name: /restore/i })).toBeInTheDocument();
  });

  it('archives an active recurring task', async () => {
    recurringTasks.mockResolvedValue([activeRecurring]);

    render(Library);
    await screen.findByDisplayValue('Daily standup');

    const section = recurringSection();
    await userEvent.click(within(section).getByRole('button', { name: /archive/i }));
    expect(archiveRecurringTask).toHaveBeenCalledWith('r1');
  });
});
