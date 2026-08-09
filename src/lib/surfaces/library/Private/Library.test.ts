import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Library from './Library.svelte';

const library = vi.hoisted(() => vi.fn());
const todayView = vi.hoisted(() => vi.fn());
const weeklyFocus = vi.hoisted(() => vi.fn());
const setHabitStrength = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const associationsFor = vi.hoisted(() => vi.fn().mockResolvedValue([]));
const achieveGoal = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock('../../../api', () => ({
  library,
  todayView,
  weeklyFocus,
  setHabitStrength,
  associationsFor,
  createValue: vi.fn().mockResolvedValue({ id: 'v1' }),
  createGoal: vi.fn().mockResolvedValue({ id: 'g1' }),
  createTask: vi.fn().mockResolvedValue({ id: 't1' }),
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
}));

const emptyLibrary = {
  values: [],
  goals: [],
  habits: [],
  tasks: [],
};

function resetApiMocks(): void {
  vi.clearAllMocks();
  library.mockResolvedValue(emptyLibrary);
  todayView.mockResolvedValue({ date: '2026-08-07', week: '2026-W32', tasks: [], habits: [] });
  weeklyFocus.mockResolvedValue({ id: 'f1', week: '2026-W32', tasks: [], createdAt: '2026-08-01' });
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
    const toolbarButtons = screen.getAllByRole('button', { name: /mark achieved/i });
    await userEvent.click(toolbarButtons[0]);
    expect(screen.getByLabelText(/goal to mark achieved/i)).toBeInTheDocument();
    expect(toolbarButtons[0]).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('button', { name: /new goal/i })).toHaveAttribute('aria-pressed', 'false');
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
    const toolbarButtons = screen.getAllByRole('button', { name: /mark achieved/i });
    await userEvent.click(toolbarButtons[0]);
    await userEvent.selectOptions(screen.getByLabelText(/goal to mark achieved/i), 'g1');
    const confirmButtons = screen.getAllByRole('button', { name: /mark achieved/i });
    await userEvent.click(confirmButtons[confirmButtons.length - 1]);
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
        },
      ],
    });

    render(Library);
    await screen.findByText('File taxes');
    for (const label of [/importance/i, /urgency/i, /deadline/i]) {
      expect(screen.getByLabelText(label)).toBeInTheDocument();
    }
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
