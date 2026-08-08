import { readFileSync, globSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const components = globSync('src/**/*.svelte').map((path) => ({
  path,
  text: readFileSync(path, 'utf8'),
}));

describe('DESIGN.md border and shadow rules', () => {
  it('pairs no 1px border with a wide shadow on the same rule', () => {
    const offenders = components
      .filter(({ text }) => /border:\s*1px[\s\S]{0,200}?box-shadow:\s*[^;]*\d{2,}px/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('DESIGN.md radius rules', () => {
  it('keeps card radius at or below 16px', () => {
    const offenders = components
      .flatMap(({ path, text }) =>
        [...text.matchAll(/border-radius:\s*(\d+)px/g)].map((match) => ({ path, px: +match[1]! })),
      )
      .filter(({ px }) => px > 16 && px < 100) // >=100px is a deliberate pill
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('DESIGN.md overlay effects', () => {
  it('uses no gradient text and no backdrop blur outside overlays', () => {
    const offenders = components
      .filter(({ path }) => !path.includes('Overlay') && !path.includes('Popover'))
      .filter(({ text }) => /backdrop-filter|background-clip:\s*text/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('DESIGN.md tone', () => {
  it('contains no gamification language anywhere in the UI', () => {
    const banned = /streak|badge|confetti|leaderboard|you're on fire|keep it up/i;
    expect(components.filter(({ text }) => banned.test(text)).map(({ path }) => path)).toEqual([]);
  });
});

describe('DESIGN.md motion', () => {
  it('guards motion behind prefers-reduced-motion via the duration tokens', () => {
    const offenders = components
      .filter(({ text }) => /transition:[^;]*\b\d{3,}ms/.test(text))
      .map(({ path }) => path);
    expect(
      offenders,
      'use var(--duration-fast) or var(--duration-state) so reduced motion applies',
    ).toEqual([]);
  });
});
