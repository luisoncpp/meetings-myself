import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import SurfaceLayoutHarness from './SurfaceLayoutHarness.svelte';

describe('SurfaceLayout', () => {
  it('renders child content', () => {
    render(SurfaceLayoutHarness);
    expect(screen.getByText('Surface content')).toBeVisible();
  });

  it('forwards aria-labelledby when provided', () => {
    const { container } = render(SurfaceLayoutHarness, { 'aria-labelledby': 'section-title' });
    expect(container.querySelector('.surface-layout')).toHaveAttribute(
      'aria-labelledby',
      'section-title',
    );
  });
});
