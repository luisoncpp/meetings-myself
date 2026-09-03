import { describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

describe('appVersion', () => {
  it('forwards to the app_version command', async () => {
    invoke.mockResolvedValue('0.1.0');
    const { appVersion } = await import('./index');
    await expect(appVersion()).resolves.toBe('0.1.0');
    expect(invoke).toHaveBeenCalledWith('app_version');
  });
});

describe('storeHealth', () => {
  it('forwards to the store_health command', async () => {
    const health = {
      status: 'setupIncomplete',
      reason: { kind: 'NoSyncFolder' },
    };
    invoke.mockResolvedValue(health);
    const { storeHealth } = await import('./index');
    await expect(storeHealth()).resolves.toEqual(health);
    expect(invoke).toHaveBeenCalledWith('store_health');
  });
});

describe('reconnectStore', () => {
  it('forwards to the reconnect_store command', async () => {
    const health = { status: 'folderMissing', path: 'G:/drive' };
    invoke.mockResolvedValue(health);
    const { reconnectStore } = await import('./index');
    await expect(reconnectStore()).resolves.toEqual(health);
    expect(invoke).toHaveBeenCalledWith('reconnect_store');
  });
});

describe('reconnect', () => {
  it('forwards to the reconnect command', async () => {
    const health = { status: 'unreadable', detail: 'WAL error' };
    invoke.mockResolvedValue(health);
    const { reconnect } = await import('./index');
    await expect(reconnect()).resolves.toEqual(health);
    expect(invoke).toHaveBeenCalledWith('reconnect');
  });
});

describe('chooseSyncFolder', () => {
  it('forwards with the folder path', async () => {
    const health = { status: 'ready' };
    invoke.mockResolvedValue(health);
    const { chooseSyncFolder } = await import('./index');
    await expect(chooseSyncFolder('/drive/sync')).resolves.toEqual(health);
    expect(invoke).toHaveBeenCalledWith('choose_sync_folder', {
      folder: '/drive/sync',
    });
  });
});

describe('syncFolder', () => {
  it('queries the configured sync folder', async () => {
    invoke.mockResolvedValue('/drive/sync');
    const { syncFolder } = await import('./index');
    await expect(syncFolder()).resolves.toEqual('/drive/sync');
    expect(invoke).toHaveBeenCalledWith('sync_folder', undefined);
  });
});

describe('setHomeZone', () => {
  it('forwards with the zone name', async () => {
    const health = { status: 'ready' };
    invoke.mockResolvedValue(health);
    const { setHomeZone } = await import('./index');
    await expect(setHomeZone('Europe/Madrid')).resolves.toEqual(health);
    expect(invoke).toHaveBeenCalledWith('set_home_zone', {
      zone: 'Europe/Madrid',
    });
  });
});
