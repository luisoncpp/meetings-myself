import { describe, expect, it, vi } from 'vitest';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

describe('api', () => {
  it('forwards appVersion to the app_version command', async () => {
    invoke.mockResolvedValue('0.1.0');
    const { appVersion } = await import('./index');
    await expect(appVersion()).resolves.toBe('0.1.0');
    expect(invoke).toHaveBeenCalledWith('app_version');
  });
});
