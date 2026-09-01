import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
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

describe('HealthBanner retry', () => {
  it('offers another open attempt when the database is unreadable', () => {
    render(HealthBanner, {
      health: { status: 'unreadable', detail: 'Invalid segment name format' },
      onretry: () => {},
    });
    expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument();
  });
});
