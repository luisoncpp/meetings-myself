import { fireEvent, render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import OrderableListHarness from './OrderableListHarness.svelte';

describe('OrderableList keyboard', () => {
  it('reorders from Alt+Arrow and announces the result', async () => {
    const onreorder = vi.fn();
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

    const row = screen.getByRole('listitem', { name: 'b' });
    row.focus();
    await userEvent.keyboard('{Alt>}{ArrowUp}{/Alt}');

    expect(onreorder).toHaveBeenCalledWith(['b', 'a', 'c']);
    expect(screen.getByRole('status')).toHaveTextContent(/b.*position 1 of 3/i);
  });

  it('moves focus between rows with bare arrow keys', async () => {
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder: vi.fn() });
    screen.getByRole('listitem', { name: 'a' }).focus();
    await userEvent.keyboard('{ArrowDown}');
    expect(screen.getByRole('listitem', { name: 'b' })).toHaveFocus();
  });
});

describe('OrderableList pointer and semantics', () => {
  it('reorders by pointer drag on the handle', async () => {
    const onreorder = vi.fn();
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

    const sourceRow = screen.getByRole('listitem', { name: 'c' });
    const targetRow = screen.getByRole('listitem', { name: 'a' });
    const handle = within(sourceRow).getByRole('button', { name: 'Reorder c' });
    await fireEvent.dragStart(handle);
    await fireEvent.dragOver(targetRow);
    await fireEvent.drop(targetRow);

    expect(onreorder).toHaveBeenCalledWith(['c', 'a', 'b']);
  });

  it('exposes list semantics with an accessible label', () => {
    render(OrderableListHarness, { items: ['a'], onreorder: vi.fn() });
    expect(screen.getByRole('list', { name: 'Reorderable list' })).toBeInTheDocument();
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
  });
});
