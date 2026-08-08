import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import App from './App.svelte';

vi.mock('./lib/api', () => ({
  storeHealth: vi.fn().mockResolvedValue({ status: 'ready' }),
  openWeeklyReviewWindow: vi.fn(),
}));

describe('App', () => {
  it('renders the app shell with the daily plan home surface', async () => {
    render(App);
    expect(await screen.findByRole('heading', { name: 'Daily Plan' })).toBeInTheDocument();
  });
});
