import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import HealthBanner from './HealthBanner.svelte';

describe('HealthBanner', () => {
  it('renders nothing when the store is ready', () => {
    const { container } = render(HealthBanner, { health: { status: 'ready' } });
    expect(container.textContent?.trim()).toBe('');
  });

  it('names the other device when the data is locked', () => {
    render(HealthBanner, {
      health: { status: 'lockedByAnotherDevice', deviceName: 'laptop', since: '2026-08-07T09:00:00Z' },
    });
    expect(screen.getByRole('alert')).toHaveTextContent(/laptop/);
  });

  it('explains a sync conflict without offering a destructive fix', () => {
    render(HealthBanner, { health: { status: 'syncConflict', artifacts: ['CURRENT (1)'] } });
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent(/CURRENT \(1\)/);
    expect(alert.textContent).not.toMatch(/delete/i);
  });

  it('tells the user what to do when the folder is missing', () => {
    render(HealthBanner, { health: { status: 'folderMissing', path: 'D:/Drive/planning' } });
    expect(screen.getByRole('alert')).toHaveTextContent(/D:\/Drive\/planning/);
  });
});

describe('HealthBanner actions', () => {
  it('lets the user pick a different folder when the configured path is gone', async () => {
    const onchooseFolder = vi.fn();
    const onretry = vi.fn();
    render(HealthBanner, {
      health: { status: 'folderMissing', path: 'G:/Drive/planning' },
      onchooseFolder,
      onretry,
    });
    await userEvent.click(screen.getByRole('button', { name: /choose a different folder/i }));
    expect(onchooseFolder).toHaveBeenCalledOnce();
    await userEvent.click(screen.getByRole('button', { name: /try again/i }));
    expect(onretry).toHaveBeenCalledOnce();
  });

  it('does not offer a folder change when another device holds the lock', () => {
    render(HealthBanner, {
      health: {
        status: 'lockedByAnotherDevice',
        deviceName: 'laptop',
        since: '2026-08-07T09:00:00Z',
      },
      onchooseFolder: vi.fn(),
      onretry: vi.fn(),
    });
    expect(screen.queryByRole('button', { name: /choose a different folder/i })).toBeNull();
  });
});
