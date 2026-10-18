import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const storeHealth = vi.hoisted(() => vi.fn());
const reconnectStore = vi.hoisted(() => vi.fn());
const pickSyncFolder = vi.hoisted(() => vi.fn());
const chooseSyncFolder = vi.hoisted(() => vi.fn());
const openCurrentReview = vi.hoisted(() => vi.fn(() => new Promise(() => {})));

vi.mock('../../api', () => ({
  storeHealth,
  reconnectStore,
  pickSyncFolder,
  chooseSyncFolder,
  todayView: vi.fn().mockResolvedValue({ date: '2026-09-03', tasks: [], habits: [] }),
  yesterdayView: vi.fn().mockResolvedValue(null),
  taskPool: vi.fn().mockResolvedValue({ ready: [], blocked: [] }),
  weeklyFocus: vi.fn().mockResolvedValue({ id: 'f', week: '2026-W36', tasks: [], createdAt: '' }),
  openCurrentReview,
  uiLanguage: vi.fn().mockResolvedValue('en'),
  setUiLanguage: vi.fn(),
}));

import AppShell from './AppShell.svelte';

beforeEach(() => {
  storeHealth.mockReset();
  reconnectStore.mockReset();
  pickSyncFolder.mockReset();
  chooseSyncFolder.mockReset();
  openCurrentReview.mockClear();
});

describe('AppShell missing folder', () => {
  it('lets the user replace a missing sync folder', async () => {
    storeHealth.mockResolvedValue({
      status: 'folderMissing',
      path: 'G:/.shortcut-targets-by-id/abc/juntas',
    });
    pickSyncFolder.mockResolvedValue('D:/Drive/planning');
    chooseSyncFolder.mockResolvedValue({
      status: 'folderMissing',
      path: 'D:/Drive/planning',
    });
    reconnectStore.mockResolvedValue({
      status: 'folderMissing',
      path: 'G:/.shortcut-targets-by-id/abc/juntas',
    });

    render(AppShell);
    await userEvent.click(
      await screen.findByRole('button', { name: /choose a different folder/i }),
    );
    expect(pickSyncFolder).toHaveBeenCalledOnce();
    expect(chooseSyncFolder).toHaveBeenCalledWith('D:/Drive/planning');
  });
});

describe('AppShell folder switching', () => {
  it('lets the user switch the folder from the navigation bar', async () => {
    storeHealth.mockResolvedValue({ status: 'ready' });
    pickSyncFolder.mockResolvedValue('D:/Drive/other-planning');
    chooseSyncFolder.mockResolvedValue({ status: 'ready' });

    render(AppShell);
    const switchButton = await screen.findByRole('button', { name: /switch folder/i });
    await userEvent.click(switchButton);

    expect(pickSyncFolder).toHaveBeenCalledOnce();
    expect(chooseSyncFolder).toHaveBeenCalledWith('D:/Drive/other-planning');
  });
});

describe('AppShell rail actions', () => {
  it('opens the Week section inside the main window', async () => {
    storeHealth.mockResolvedValue({ status: 'ready' });

    render(AppShell);
    await userEvent.click(await screen.findByRole('button', { name: 'Week' }));

    expect(openCurrentReview).toHaveBeenCalledOnce();
    expect(screen.getByRole('button', { name: 'Week' })).toHaveAttribute('aria-current', 'page');
  });

  it('does nothing if folder picking is cancelled', async () => {
    storeHealth.mockResolvedValue({ status: 'ready' });
    pickSyncFolder.mockResolvedValue(null);

    render(AppShell);
    const switchButton = await screen.findByRole('button', { name: /switch folder/i });
    await userEvent.click(switchButton);

    expect(pickSyncFolder).toHaveBeenCalledOnce();
    expect(chooseSyncFolder).not.toHaveBeenCalled();
  });
});
