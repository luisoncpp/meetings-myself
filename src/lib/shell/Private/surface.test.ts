import { describe, expect, it } from 'vitest';
import { currentSurface } from './surface';

describe('currentSurface', () => {
  it('defaults to the main window', () => {
    expect(currentSurface('')).toBe('main');
    expect(currentSurface('?other=1')).toBe('main');
  });

  it('recognises the weekly review window', () => {
    expect(currentSurface('?surface=weekly-review')).toBe('weekly-review');
  });

  it('ignores an unknown surface rather than rendering nothing', () => {
    expect(currentSurface('?surface=nonsense')).toBe('main');
  });
});
