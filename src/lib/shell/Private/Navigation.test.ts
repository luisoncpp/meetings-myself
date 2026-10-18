import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

vi.mock('../../api', () => ({
  uiLanguage: vi.fn().mockResolvedValue('en'),
  setUiLanguage: vi.fn(),
}));

import Navigation from './Navigation.svelte';

describe('Navigation', () => {
  it('switches between Today, Library and Week in the same window', async () => {
    const onnavigate = vi.fn();
    render(Navigation, { current: 'today', onnavigate });

    await userEvent.click(screen.getByRole('button', { name: 'Library' }));
    expect(onnavigate).toHaveBeenCalledWith('library');

    await userEvent.click(screen.getByRole('button', { name: 'Week' }));
    expect(onnavigate).toHaveBeenCalledWith('week');

    await userEvent.click(screen.getByRole('button', { name: 'Today' }));
    expect(onnavigate).toHaveBeenCalledWith('today');
  });

  it('marks the current section', () => {
    render(Navigation, { current: 'library', onnavigate: vi.fn() });
    expect(screen.getByRole('button', { name: 'Library' })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('button', { name: 'Today' })).not.toHaveAttribute('aria-current');
  });

  it('calls onswitchFolder when switch folder button is clicked', async () => {
    const onswitchFolder = vi.fn();
    render(Navigation, { current: 'today', onnavigate: vi.fn(), onswitchFolder });

    await userEvent.click(screen.getByRole('button', { name: /switch folder/i }));
    expect(onswitchFolder).toHaveBeenCalledOnce();
  });

  it('does not render switch folder button when onswitchFolder is not provided', () => {
    render(Navigation, { current: 'today', onnavigate: vi.fn() });
    expect(screen.queryByRole('button', { name: /switch folder/i })).toBeNull();
  });
});
