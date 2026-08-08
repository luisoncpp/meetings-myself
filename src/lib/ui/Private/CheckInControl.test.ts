import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import CheckInControl from './CheckInControl.svelte';

describe('CheckInControl', () => {
  it('offers exactly the three agreed outcomes and no others', () => {
    render(CheckInControl, { value: null, label: 'Writing practice', onchange: vi.fn() });
    const options = screen.getAllByRole('radio');
    expect(options.map((option) => option.getAttribute('aria-label'))).toEqual([
      'Done',
      'Skipped',
      'Not completed',
    ]);
  });

  it('marks the current outcome and reports a change', async () => {
    const onchange = vi.fn();
    render(CheckInControl, { value: 'done', label: 'Writing practice', onchange });

    expect(screen.getByRole('radio', { name: 'Done' })).toBeChecked();
    await userEvent.click(screen.getByRole('radio', { name: 'Skipped' }));
    expect(onchange).toHaveBeenCalledWith('skipped');
  });

  it('is reachable and operable from the keyboard', async () => {
    const onchange = vi.fn();
    render(CheckInControl, { value: 'done', label: 'Writing practice', onchange });

    await userEvent.tab();
    expect(screen.getByRole('radio', { name: 'Done' })).toHaveFocus();
    await userEvent.keyboard('{ArrowRight}');
    expect(onchange).toHaveBeenCalledWith('skipped');
  });

  it('names the habit it belongs to, so the group is unambiguous', () => {
    render(CheckInControl, { value: null, label: 'Writing practice', onchange: vi.fn() });
    expect(screen.getByRole('radiogroup', { name: /Writing practice/ })).toBeInTheDocument();
  });
});
