import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Setup from './Setup.svelte';

const chooseSyncFolder = vi.hoisted(() => vi.fn());
const setHomeZone = vi.hoisted(() => vi.fn());
const pickSyncFolder = vi.hoisted(() => vi.fn());
const availableTimeZones = vi.hoisted(() => vi.fn());

vi.mock('../../../api', () => ({
  chooseSyncFolder,
  setHomeZone,
  pickSyncFolder,
  availableTimeZones,
}));

function mockFolderThenZoneFlow(): void {
  chooseSyncFolder.mockResolvedValue({
    status: 'setupIncomplete',
    reason: { kind: 'NoHomeZone' },
  });
  setHomeZone.mockResolvedValue({ status: 'ready' });
  pickSyncFolder.mockResolvedValue('D:/Drive/self-planning');
}

describe('Setup', () => {
  beforeEach(() => {
    chooseSyncFolder.mockReset();
    setHomeZone.mockReset();
    pickSyncFolder.mockReset();
    availableTimeZones.mockReset();
    availableTimeZones.mockResolvedValue(['Europe/Madrid', 'UTC']);
  });

  it('walks from no folder, to no zone, to ready', async () => {
    mockFolderThenZoneFlow();
    const onready = vi.fn();
    render(Setup, {
      health: { status: 'setupIncomplete', reason: { kind: 'NoSyncFolder' } },
      onready,
    });

    await userEvent.click(screen.getByRole('button', { name: /choose folder/i }));
    expect(chooseSyncFolder).toHaveBeenCalledWith('D:/Drive/self-planning');

    await userEvent.type(screen.getByLabelText(/home time zone/i), 'Europe/Madrid');
    await userEvent.click(screen.getByRole('button', { name: /finish setup/i }));
    expect(setHomeZone).toHaveBeenCalledWith('Europe/Madrid');
    expect(onready).toHaveBeenCalled();
  });

  it('explains why the time zone matters instead of just asking for it', () => {
    render(Setup, {
      health: { status: 'setupIncomplete', reason: { kind: 'NoHomeZone' } },
      onready: vi.fn(),
    });

    expect(screen.getByText(/every device/i)).toBeInTheDocument();
  });
});

describe('Setup when the engine refuses the folder', () => {
  beforeEach(() => {
    chooseSyncFolder.mockReset();
    pickSyncFolder.mockReset();
    availableTimeZones.mockReset();
    availableTimeZones.mockResolvedValue(['Europe/Madrid', 'UTC']);
  });

  it('leaves setup when the chosen folder cannot be opened', async () => {
    chooseSyncFolder.mockResolvedValue({
      status: 'unreadable',
      detail: 'WAL error',
    });
    pickSyncFolder.mockResolvedValue('D:/Drive/self-planning');
    const onready = vi.fn();
    render(Setup, {
      health: { status: 'setupIncomplete', reason: { kind: 'NoSyncFolder' } },
      onready,
    });

    await userEvent.click(screen.getByRole('button', { name: /choose folder/i }));
    expect(onready).toHaveBeenCalled();
  });
});
