import { globSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

function readAll(pattern: string): Array<{ path: string; text: string }> {
  return globSync(pattern, { exclude: (p) => p.includes('target') })
    .map((path) => ({ path, text: readFileSync(path, 'utf8') }));
}

describe('time handling', () => {
  // ADR 0001: all dates use the synchronized home time zone, never the device zone.
  it('never uses the device local time zone in Rust', () => {
    const offenders = readAll('{crates,src-tauri,launcher}/**/*.rs')
      .filter(({ text }) => /chrono::Local|Local::now\(\)/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });

  // Only SystemClock may read the wall clock; everything else takes a Clock.
  it('only SystemClock calls Utc::now', () => {
    const offenders = readAll('{crates,src-tauri,launcher}/**/*.rs')
      .filter(({ path }) => !path.endsWith('system_clock.rs'))
      .filter(({ text }) => /Utc::now\(\)/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('crate boundaries', () => {
  // 0001: binaries depend on planning-app only.
  it.each(['src-tauri', 'launcher'])('%s depends on planning-app only', (binary) => {
    const manifests = readAll(`${binary}/Cargo.toml`);
    if (manifests.length === 0) return; // launcher arrives in plan 0008
    const forbidden = ['planning-core', 'planning-store', 'planning-reports', 'surrealdb'];
    const text = manifests[0]!.text;
    expect(forbidden.filter((dep) => text.includes(dep))).toEqual([]);
  });

  // GUIDELINES.md: a deep module's internals are reachable only through its interface.
  it('nothing outside a crate names its private module', () => {
    const offenders = readAll('{src-tauri,launcher}/**/*.rs')
      .filter(({ text }) => /planning_(core|store|app|reports)::private/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('domain invariants', () => {
  // ADR 0002: nothing is ever hard-deleted.
  it('contains no DELETE statement outside test code', () => {
    const offenders = readAll('crates/**/*.rs')
      .filter(({ path }) => !path.includes('test'))
      .filter(({ text }) => /\bDELETE\b/.test(text.replace(/#\[cfg\(test\)\][\s\S]*$/, '')))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});

describe('frontend boundaries', () => {
  // Only the api deep module may reach Tauri IPC.
  it('only src/lib/api/Private reaches @tauri-apps/api', () => {
    const offenders = readAll('src/**/*.{ts,svelte}')
      .filter(({ path }) => !path.replace(/\\/g, '/').startsWith('src/lib/api/Private/'))
      .filter(({ text }) => text.includes('@tauri-apps/api'))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });

  // DESIGN.md: components style from tokens, never raw hex.
  it('has no raw hex colours outside tokens.css', () => {
    const offenders = readAll('src/**/*.svelte')
      .filter(({ text }) => /#[0-9a-fA-F]{3,8}\b/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });
});
