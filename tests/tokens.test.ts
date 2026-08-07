import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync('src/styles/tokens.css', 'utf8');

function token(name: string): string {
  const match = css.match(new RegExp(`--${name}:\\s*(#[0-9A-Fa-f]{6})`));
  if (!match) throw new Error(`token --${name} is not defined in tokens.css`);
  return match[1]!;
}

function luminance(hex: string): number {
  const channels = [1, 3, 5]
    .map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * channels[0]! + 0.7152 * channels[1]! + 0.0722 * channels[2]!;
}

function contrast(foreground: string, background: string): number {
  const [lighter, darker] = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (lighter! + 0.05) / (darker! + 0.05);
}

// Every pair that renders text or a focus ring must clear WCAG 2.1 AA (4.5:1).
const textPairs: ReadonlyArray<[string, string]> = [
  ['color-ink', 'color-base'],
  ['color-ink', 'color-lift'],
  ['color-ink', 'color-raised'],
  ['color-ink-muted', 'color-base'],
  ['color-ink-muted', 'color-lift'],
  ['color-ink-muted', 'color-raised'],
  ['color-gold', 'color-base'],
  ['color-gold', 'color-lift'],
  ['color-gold-deep', 'color-base'],
  ['color-overdue', 'color-lift'],
  ['color-done', 'color-lift'],
  ['color-base', 'color-gold'],
];

describe('design tokens', () => {
  it.each(textPairs)('%s on %s meets WCAG AA', (foreground, background) => {
    expect(contrast(token(foreground), token(background))).toBeGreaterThanOrEqual(4.5);
  });

  it('defines the fixed type scale without fluid clamp', () => {
    expect(css).toMatch(/--text-display:\s*1\.75rem/);
    expect(css).not.toMatch(/clamp\(/);
  });
});
