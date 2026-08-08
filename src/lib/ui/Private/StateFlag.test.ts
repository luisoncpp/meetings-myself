import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StateFlag from './StateFlag.svelte';

describe('StateFlag', () => {
  it.each([
    ['archived', 'Archived'],
    ['overdue', 'Overdue'],
    ['unpinned', 'Unpinned'],
    ['completed', 'Completed'],
  ] as const)('renders visible text for %s', (kind, text) => {
    render(StateFlag, { kind });
    expect(screen.getByText(text)).toBeVisible();
  });
});
