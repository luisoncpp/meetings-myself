import { fireEvent, render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { localeStore } from '../../../i18n';
import Library from './Library.svelte';
import * as api from './library-test-api';
import { activeRecurring, archivedRecurring, task, withEntities, writingHabit } from './library-test-api';

vi.mock('../../../api', () => import('./library-test-api'));

async function openTab(name: string): Promise<void> {
  await userEvent.click(await screen.findByRole('tab', { name }));
}

beforeEach(() => {
  api.resetLibraryApi();
  void localeStore.setLocale('en', /*persist=*/ false);
});

describe('Library tasks tab rows', () => {
  it('lists active tasks one per line, with archived ones kept out', async () => {
    withEntities({ tasks: [task('t1', 'Send invoice'), task('t2', 'Old', { archived: true })] });
    render(Library);

    expect(await screen.findByText('Send invoice')).toBeInTheDocument();
    expect(screen.queryByText('Old')).toBeNull();
    expect(screen.getByRole('tab', { name: 'Tasks' })).toHaveAttribute('aria-selected', 'true');
  });

  it("marks this week's focus and next week's focus differently", async () => {
    withEntities({ tasks: [task('t-this', 'Draft post'), task('t-next', 'Plan trip')] });
    render(Library);

    const thisWeek = await screen.findByRole('group', { name: 'Draft post' });
    const nextWeek = screen.getByRole('group', { name: 'Plan trip' });
    expect(within(thisWeek).getByText("In this week's focus")).toBeInTheDocument();
    expect(within(nextWeek).getByText("In next week's focus")).toBeInTheDocument();
  });

  it('completes a one-off from its row', async () => {
    withEntities({ tasks: [task('t1', 'Send invoice')] });
    render(Library);

    await userEvent.click(await screen.findByRole('checkbox', { name: /mark done: send invoice/i }));
    expect(api.completeTask).toHaveBeenCalledWith('t1', undefined);
  });
});

describe('Library tasks tab editing', () => {
  it('opens every editable field in the detail panel', async () => {
    withEntities({ tasks: [task('t1', 'Send invoice')] });
    render(Library);

    await userEvent.click(await screen.findByRole('button', { name: 'Send invoice' }));
    const panel = screen.getByRole('complementary', { name: 'Send invoice' });
    expect(within(panel).getByLabelText('Title')).toHaveValue('Send invoice');
    expect(within(panel).getByLabelText('Importance')).toBeInTheDocument();
    expect(within(panel).getByLabelText('Urgency')).toBeInTheDocument();
    expect(within(panel).getByLabelText('Deadline')).toBeInTheDocument();

    await userEvent.click(within(panel).getByRole('checkbox', { name: 'One-off' }));
    expect(api.setTaskOneOff).toHaveBeenCalledWith('t1', false);
  });

  it("adds a task to next week's focus from the right-click menu", async () => {
    withEntities({ tasks: [task('t1', 'Send invoice')] });
    render(Library);

    await fireEvent.contextMenu(await screen.findByRole('group', { name: 'Send invoice' }));
    await userEvent.click(screen.getByRole('menuitem', { name: "Add to next week's focus" }));
    expect(api.addToFocus).toHaveBeenCalledWith('2026-W33', 't1');
  });

  it('creates a task with one-off unchecked when requested', async () => {
    render(Library);
    await userEvent.click(await screen.findByRole('button', { name: /new task/i }));
    await userEvent.type(screen.getByLabelText(/task name/i), 'Pay rent');
    await userEvent.click(screen.getByRole('checkbox', { name: /one-off/i }));
    await userEvent.click(screen.getByRole('button', { name: /create task/i }));
    expect(api.createTask).toHaveBeenCalledWith('Pay rent', false);
  });
});

describe('Library archive tab grouping', () => {
  it('groups archived tasks into done and not done', async () => {
    withEntities({
      tasks: [
        task('t1', 'Sent invoice', { archived: true, state: 'archived', completedOn: '2026-08-05' }),
        task('t2', 'Dropped idea', { archived: true, state: 'archived' }),
      ],
    });
    render(Library);
    await openTab('Archive');

    const done = screen.getByRole('heading', { name: 'Done' }).nextElementSibling as HTMLElement;
    const notDone = screen.getByRole('heading', { name: 'Not done' }).nextElementSibling as HTMLElement;
    expect(within(done).getByText('Sent invoice')).toBeInTheDocument();
    expect(within(done).getByText('2026-08-05')).toBeInTheDocument();
    expect(within(notDone).getByText('Dropped idea')).toBeInTheDocument();
  });
});

describe('Library archive tab restore', () => {
  it('restores an archived task', async () => {
    withEntities({ tasks: [task('t2', 'Dropped idea', { archived: true, state: 'archived' })] });
    render(Library);
    await openTab('Archive');

    await userEvent.click(screen.getByRole('button', { name: 'Restore' }));
    expect(api.restoreEntity).toHaveBeenCalledWith({ kind: 'task', id: 't2' });
  });

  it('lists archived habits and recurring rules too', async () => {
    withEntities({ habits: [{ ...writingHabit, archived: true }] });
    api.recurringTasks.mockResolvedValue([activeRecurring, archivedRecurring]);
    render(Library);
    await openTab('Archive');

    expect(screen.getByText('Writing')).toBeInTheDocument();
    expect(screen.getByText('Old rule')).toBeInTheDocument();
    expect(screen.queryByText('Daily standup')).toBeNull();
  });
});

describe('Library delete guard', () => {
  it('never offers to delete anything', async () => {
    withEntities({ tasks: [task('t1', 'Send invoice')], habits: [writingHabit] });
    render(Library);
    await screen.findByText('Send invoice');
    expect(screen.queryByRole('button', { name: /delete/i })).toBeNull();
  });
});

