import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const storeHealth = vi.hoisted(() => vi.fn());
const reconnectStore = vi.hoisted(() => vi.fn());
const pickSyncFolder = vi.hoisted(() => vi.fn());
const chooseSyncFolder = vi.hoisted(() => vi.fn());

vi.mock('../../api', () => ({
  storeHealth,
  reconnectStore,
  pickSyncFolder,
  chooseSyncFolder,
}));

import AppShell from './AppShell.svelte';

describe('AppShell', () => {
  beforeEach(() => {
    storeHealth.mockReset();
    reconnectStore.mockReset();
    pickSyncFolder.mockReset();
    chooseSyncFolder.mockReset();
  });

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

    render(AppShell, { surface: 'main' });
    await userEvent.click(
      await screen.findByRole('button', { name: /choose a different folder/i }),
    );
    expect(pickSyncFolder).toHaveBeenCalledOnce();
    expect(chooseSyncFolder).toHaveBeenCalledWith('D:/Drive/planning');
  });
});
