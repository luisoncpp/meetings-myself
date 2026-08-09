import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ListRowHarness from './ListRowHarness.svelte';

describe('ListRow', () => {
  it('renders children inside a raised surface row', () => {
    const { container } = render(ListRowHarness);
    const row = container.querySelector('.row');
    expect(row).not.toBeNull();
    expect(row?.textContent).toContain('Row content');
  });
});
