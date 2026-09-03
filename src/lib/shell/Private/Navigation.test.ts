import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

const openWeeklyReviewWindow = vi.hoisted(() => vi.fn());

vi.mock('../../api', () => ({
  openWeeklyReviewWindow,
  uiLanguage: vi.fn().mockResolvedValue('en'),
  setUiLanguage: vi.fn(),
}));

import Navigation from './Navigation.svelte';

describe('Navigation', () => {
  it('navigates between views when clicked', async () => {
    const onnavigate = vi.fn();
    render(Navigation, { current: 'daily-plan', onnavigate });

    await userEvent.click(screen.getByRole('button', { name: /library/i }));
    expect(onnavigate).toHaveBeenCalledWith('library');

    await userEvent.click(screen.getByRole('button', { name: /daily plan/i }));
    expect(onnavigate).toHaveBeenCalledWith('daily-plan');
  });

  it('opens weekly review window', async () => {
    render(Navigation, { current: 'daily-plan', onnavigate: vi.fn() });
    await userEvent.click(
      screen.getByRole('button', { name: /open weekly review/i }),
    );
    expect(openWeeklyReviewWindow).toHaveBeenCalledOnce();
  });

  it('calls onswitchFolder when switch folder button is clicked', async () => {
    const onswitchFolder = vi.fn();
    render(Navigation, {
      current: 'daily-plan',
      onnavigate: vi.fn(),
      onswitchFolder,
    });

    const switchBtn = screen.getByRole('button', { name: /switch folder/i });
    expect(switchBtn).toBeInTheDocument();
    await userEvent.click(switchBtn);
    expect(onswitchFolder).toHaveBeenCalledOnce();
  });

  it('does not render switch folder button when onswitchFolder is not provided', () => {
    render(Navigation, { current: 'daily-plan', onnavigate: vi.fn() });
    expect(
      screen.queryByRole('button', { name: /switch folder/i }),
    ).toBeNull();
  });
});
