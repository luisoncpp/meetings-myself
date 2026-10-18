import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import App from './App.svelte';

vi.mock('./lib/api', () => ({
  storeHealth: vi.fn().mockResolvedValue({ status: 'ready' }),
  reconnectStore: vi.fn().mockResolvedValue({ status: 'ready' }),
  pickSyncFolder: vi.fn(),
  chooseSyncFolder: vi.fn(),
  uiLanguage: vi.fn().mockResolvedValue('en'),
  setUiLanguage: vi.fn().mockResolvedValue(undefined),
  weeklyFocus: vi.fn().mockResolvedValue({ id: 'f', week: '2026-W32', tasks: [], createdAt: '' }),
  yesterdayView: vi.fn().mockResolvedValue({ date: '2026-08-06', week: '2026-W32', tasks: [], habits: [] }),
  todayView: vi.fn().mockResolvedValue({
    date: '2026-08-07',
    week: '2026-W32',
    tasks: [],
    habits: [],
  }),
  taskPool: vi.fn().mockResolvedValue({ focus: [], rest: [] }),
}));

describe('App', () => {
  it('renders the app shell with the Today section as home', async () => {
    render(App);
    expect(await screen.findByRole('heading', { name: 'Today' })).toBeInTheDocument();
  });
});
