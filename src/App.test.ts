import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import App from './App.svelte';

describe('App', () => {
  it('renders the Daily Plan as the home surface', () => {
    render(App);
    expect(screen.getByRole('heading', { name: 'Daily Plan' })).toBeInTheDocument();
  });
});
