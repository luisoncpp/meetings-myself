import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { localeStore } from '../../../i18n';
import Library from './Library.svelte';
import * as api from './library-test-api';
import { activeRecurring, withEntities, writingHabit } from './library-test-api';

vi.mock('../../../api', () => import('./library-test-api'));

async function openTab(name: string): Promise<void> {
  await userEvent.click(await screen.findByRole('tab', { name }));
}

beforeEach(() => {
  api.resetLibraryApi();
  void localeStore.setLocale('en', /*persist=*/ false);
});

describe('Library habit details', () => {
  it('renames a habit and sets strength by hand in its detail panel', async () => {
    withEntities({ habits: [writingHabit] });
    render(Library);
    await openTab('Habits');
    await userEvent.click(screen.getByRole('button', { name: 'Writing' }));

    const title = screen.getByLabelText('Habit title');
    await userEvent.clear(title);
    await userEvent.type(title, 'Evening pages');
    await userEvent.tab();
    expect(api.renameHabit).toHaveBeenCalledWith('h1', 'Evening pages');

    await userEvent.selectOptions(screen.getByLabelText(/habit strength/i), 'strengthening');
    expect(api.setHabitStrength).toHaveBeenCalledWith('h1', 'strengthening');
    expect(screen.queryByRole('progressbar')).toBeNull();
  });

  it('uses a labelled switch instead of a pin to keep a habit out of daily plans', async () => {
    withEntities({ habits: [writingHabit] });
    render(Library);
    await openTab('Habits');
    await userEvent.click(screen.getByRole('button', { name: 'Writing' }));

    await userEvent.click(screen.getByRole('switch', { name: 'Show in daily plan' }));
    expect(api.setHabitPinned).toHaveBeenCalledWith('h1', false);
  });
});

describe('Library habit list and creation', () => {
  it('flags habits that are not added to daily plans', async () => {
    withEntities({ habits: [{ ...writingHabit, pinned: false }] });
    render(Library);
    await openTab('Habits');
    expect(screen.getByText('Not daily')).toBeInTheDocument();
  });

  it('requires a cadence before a habit can be created', async () => {
    render(Library);
    await openTab('Habits');
    await userEvent.click(screen.getByRole('button', { name: /new habit/i }));
    await userEvent.type(screen.getByLabelText(/habit name/i), 'Meditation');

    expect(screen.getByRole('button', { name: /create habit/i })).toBeDisabled();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Monday' }));
    expect(screen.getByRole('button', { name: /create habit/i })).toBeEnabled();
  });

  it('shows habit strength and weekday labels in Spanish', async () => {
    await localeStore.setLocale('es', /*persist=*/ false);
    withEntities({ habits: [writingHabit] });
    render(Library);
    await openTab('Hábitos');
    await userEvent.click(screen.getByRole('button', { name: 'Writing' }));

    expect(screen.getByLabelText(/fortaleza del hábito/i)).toHaveTextContent(
      'Dependiente del recordatorio',
    );
    expect(screen.getByRole('checkbox', { name: 'Lunes' })).toBeInTheDocument();
  });
});

describe('Library goals tab', () => {
  it('marks a goal achieved from its detail panel', async () => {
    withEntities({
      goals: [{ id: 'g1', title: 'Ship it', achieved: false, archived: false, targetDate: null }],
    });
    render(Library);
    await openTab('Goals');
    await userEvent.click(screen.getByRole('button', { name: 'Ship it' }));
    await userEvent.click(screen.getByRole('button', { name: 'Mark achieved' }));
    expect(api.achieveGoal).toHaveBeenCalledWith('g1');
  });
});

describe('Library recurring tab', () => {
  it('creates a recurring task with daily recurrence', async () => {
    const created = { ...activeRecurring, id: 'r-new', title: 'Water plants' };
    api.recurringTasks.mockResolvedValueOnce([]).mockResolvedValue([created]);
    render(Library);
    await openTab('Recurring');

    await userEvent.click(screen.getByRole('button', { name: /new recurring task/i }));
    await userEvent.type(screen.getByLabelText(/recurring task title/i), 'Water plants');
    await userEvent.click(screen.getByRole('button', { name: /^create$/i }));

    expect(api.createRecurringTask).toHaveBeenCalledWith('Water plants', { kind: 'daily' });
    expect(await screen.findByDisplayValue('Water plants')).toBeInTheDocument();
  });

  it('archives an active rule', async () => {
    api.recurringTasks.mockResolvedValue([activeRecurring]);
    render(Library);
    await openTab('Recurring');
    await userEvent.click(screen.getByRole('button', { name: 'Archive' }));
    expect(api.archiveRecurringTask).toHaveBeenCalledWith('r1');
  });
});
