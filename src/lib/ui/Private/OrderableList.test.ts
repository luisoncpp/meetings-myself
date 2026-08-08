import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import OrderableListHarness from './OrderableListHarness.svelte';

describe('OrderableList', () => {
  it('reorders from the keyboard and announces the result', async () => {
    const onreorder = vi.fn();
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

    const row = screen.getByRole('option', { name: 'b' });
    row.focus();
    await userEvent.keyboard('{Alt>}{ArrowUp}{/Alt}');

    expect(onreorder).toHaveBeenCalledWith(['b', 'a', 'c']);
    expect(screen.getByRole('status')).toHaveTextContent(/b.*position 1 of 3/i);
  });

  it('moves focus between rows with bare arrow keys', async () => {
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder: vi.fn() });
    screen.getByRole('option', { name: 'a' }).focus();
    await userEvent.keyboard('{ArrowDown}');
    expect(screen.getByRole('option', { name: 'b' })).toHaveFocus();
  });

  it('reorders by pointer drag', async () => {
    const onreorder = vi.fn();
    render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

    const source = screen.getByRole('option', { name: 'c' });
    const target = screen.getByRole('option', { name: 'a' });
    await fireEvent.dragStart(source);
    await fireEvent.dragOver(target);
    await fireEvent.drop(target);

    expect(onreorder).toHaveBeenCalledWith(['c', 'a', 'b']);
  });
});
