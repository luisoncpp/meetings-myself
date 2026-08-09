import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import InputHarness from './InputHarness.svelte';
import Input from './Input.svelte';

describe('Input', () => {
  it('associates Field label with input via forId', () => {
    render(InputHarness);
    const input = screen.getByLabelText('Task name');
    expect(input).toBeVisible();
    expect(input).toHaveAttribute('id', 'task-input');
  });

  it('renders with aria-label when provided', () => {
    render(Input, { 'aria-label': 'Deadline' });
    expect(screen.getByLabelText('Deadline')).toBeVisible();
  });
});
