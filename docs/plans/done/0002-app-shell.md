# App Shell & Design Tokens — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](../0001-self-planning-app.md) first — its **Global Constraints**
> section applies to every task here and is not repeated.

**Goal:** Stand up a Tauri 2 + Svelte 5 desktop shell with a Cargo workspace, both test harnesses
green, resolved design tokens, and automated guards for the architecture rules the rest of the plan
set depends on.

**Architecture:** A Cargo workspace at the repo root holds four library crates plus two binaries;
this plan creates the workspace and the Tauri binary, leaving the library crates for plans
0003–0006. The Svelte 5 frontend is a plain Vite app — no SvelteKit, because a three-surface
desktop app needs no routing or SSR. Quality gates (`npm run check`) run Rust and TypeScript
checks together so no task can pass with half the codebase unverified.

**Tech Stack:** Tauri 2.11, Rust 1.95 (edition 2021), Svelte 5.56 (runes), Vite 8.2, TypeScript
7.0, Vitest 4.1 + @testing-library/svelte 5.4 + jsdom, ESLint 10, Prettier 3.9, fallow 3.14.

---

## Global constraints

See [0001-self-planning-app.md](../0001-self-planning-app.md#global-constraints). Additionally, for
this plan only:

- Pin **exact** versions (no `^`) for `svelte`, `vite`, `vitest`, and the Tauri crates. A shell is
  a foundation; silent minor drift here breaks every downstream plan.
- The Tauri binary contains **no domain logic**. Until plan 0004 exists it exposes exactly one
  command, `app_version`, purely to prove the IPC bridge works.

---

## File structure

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Workspace root: members, shared dependency versions |
| `src-tauri/Cargo.toml` | Tauri binary manifest |
| `src-tauri/tauri.conf.json` | Window config, build hooks, CSP |
| `src-tauri/build.rs` | `tauri_build::build()` |
| `src-tauri/src/main.rs` | Entry point — calls `lib::run()`, nothing else |
| `src-tauri/src/lib.rs` | Public shell interface: builder wiring + command registration |
| `src-tauri/src/private/commands.rs` | The `app_version` command |
| `package.json` | Scripts, pinned frontend deps |
| `vite.config.ts` | Dev server on 1420, ignores Rust dirs |
| `vitest.config.ts` | jsdom environment, setup file |
| `tsconfig.json` | Strict TS for `src/` |
| `index.html` | Vite entry |
| `src/main.ts` | Mounts `App.svelte` |
| `src/App.svelte` | Shell placeholder — replaced wholesale by plan 0007 |
| `src/styles/tokens.css` | Resolved design tokens |
| `src/lib/api/index.ts` | Deep-module public interface over `invoke` |
| `src/lib/api/Private/bridge.ts` | The only file that imports `@tauri-apps/api/core` |
| `tests/setup.ts` | jest-dom matchers |
| `tests/tokens.test.ts` | WCAG contrast guard |
| `tests/architecture.test.ts` | Boundary + forbidden-API guards |
| `eslint.config.js` | Flat config |
| `fallow.json` | fallow boundaries + entry points |
| `docs/architecture/app-shell.md` | New architecture doc |

---

### Task 1: Frontend scaffold with a passing component test

**Files:**
- Create: `package.json`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `tsconfig.node.json`,
  `index.html`, `src/main.ts`, `src/App.svelte`, `.gitignore`
- Test: `tests/setup.ts`, `src/App.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces: `npm run test` (Vitest), `npm run build` (Vite → `dist/`), `npm run typecheck`
  (`svelte-check`). Plan 0007 relies on the Vitest + jsdom + `@testing-library/svelte` setup
  established here.

- [ ] **Step 1: Create `package.json` with exact pins**

```json
{
  "name": "self-planning",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "typecheck": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "eslint .",
    "format": "prettier --write .",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "7.2.0",
    "@tauri-apps/cli": "2.11.4",
    "@testing-library/jest-dom": "6.9.1",
    "@testing-library/svelte": "5.4.2",
    "@vitest/coverage-v8": "4.1.10",
    "eslint": "10.8.0",
    "eslint-plugin-svelte": "3.14.0",
    "jsdom": "30.0.1",
    "prettier": "3.9.6",
    "prettier-plugin-svelte": "3.4.0",
    "svelte": "5.56.8",
    "svelte-check": "4.7.4",
    "typescript": "7.0.2",
    "typescript-eslint": "8.47.0",
    "vite": "8.2.1",
    "vitest": "4.1.10"
  },
  "dependencies": {
    "@tauri-apps/api": "2.11.1"
  }
}
```

If `npm install` reports that a pinned version does not exist, install the nearest published
version of the same minor and record the substitution in the commit message. Do not switch to
range specifiers.

- [ ] **Step 2: Create the Vite and Vitest configs**

`vite.config.ts`:

```ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri serves the frontend from a fixed port and watches Rust separately.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/launcher/**'] },
  },
  build: { target: 'chrome110', sourcemap: true },
});
```

`vitest.config.ts`:

```ts
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte(/*options=*/ { hot: false })],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['tests/setup.ts'],
    include: ['src/**/*.test.ts', 'tests/**/*.test.ts'],
  },
});
```

- [ ] **Step 3: Create `tsconfig.json` and `tsconfig.node.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "types": ["vitest/globals", "@testing-library/jest-dom"]
  },
  "include": ["src/**/*.ts", "src/**/*.svelte", "tests/**/*.ts"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

`tsconfig.node.json`:

```json
{
  "compilerOptions": {
    "composite": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "types": ["node"]
  },
  "include": ["vite.config.ts", "vitest.config.ts"]
}
```

- [ ] **Step 4: Create `tests/setup.ts`**

```ts
import '@testing-library/jest-dom/vitest';
```

- [ ] **Step 5: Write the failing component test**

`src/App.test.ts`:

```ts
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import App from './App.svelte';

describe('App', () => {
  it('renders the Daily Plan as the home surface', () => {
    render(App);
    expect(screen.getByRole('heading', { name: 'Daily Plan' })).toBeInTheDocument();
  });
});
```

- [ ] **Step 6: Run the test to verify it fails**

```bash
npm install && npm run test
```

Expected: FAIL — `Failed to resolve import "./App.svelte"`.

- [ ] **Step 7: Create `index.html`, `src/main.ts`, and `src/App.svelte`**

`index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Self-Planning</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

`src/main.ts`:

```ts
import { mount } from 'svelte';
import App from './App.svelte';
import './styles/tokens.css';

export default mount(App, { target: document.getElementById('app')! });
```

`src/App.svelte` — a deliberate placeholder; plan 0007 replaces its body entirely:

```svelte
<script lang="ts">
  // Shell placeholder. Plan 0007 replaces this with the real surfaces.
</script>

<main>
  <h1>Daily Plan</h1>
</main>

<style>
  main {
    min-height: 100vh;
    padding: var(--space-6);
    background: var(--color-base);
    color: var(--color-ink);
    font-family: var(--font-sans);
    font-size: var(--text-body);
  }

  h1 {
    font-size: var(--text-display);
    font-weight: 600;
    line-height: 1.15;
  }
</style>
```

Create `src/styles/tokens.css` as an empty file for now — Task 3 fills it. `src/main.ts` imports
it, so it must exist.

- [ ] **Step 8: Run the test to verify it passes**

```bash
npm run test
```

Expected: PASS — 1 test.

- [ ] **Step 9: Create `.gitignore`**

```gitignore
node_modules/
dist/
target/
.vite/
coverage/
*.local
```

- [ ] **Step 10: Commit**

```bash
git add package.json package-lock.json vite.config.ts vitest.config.ts tsconfig.json tsconfig.node.json index.html src tests .gitignore
git commit -m "feat: scaffold Svelte 5 + Vite frontend with Vitest harness"
```

---

### Task 2: Cargo workspace and Tauri 2 shell

**Files:**
- Create: `Cargo.toml`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`,
  `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/private/mod.rs`,
  `src-tauri/src/private/commands.rs`, `src-tauri/icons/` (generated)
- Modify: `package.json` (add `check:rust` script)
- Test: `src-tauri/src/private/commands.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Consumes: `npm run build` from Task 1 (Tauri's `beforeBuildCommand`).
- Produces:
  - Workspace `[workspace.dependencies]` table that plans 0003–0006 add crates to.
  - `app_version() -> String` Tauri command, invocable from the frontend as `app_version`.
  - `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` both green.

- [ ] **Step 1: Create the workspace root `Cargo.toml`**

Members for crates that do not exist yet are added by their own plans. This file lists only what
exists now, plus the shared dependency table later plans extend.

```toml
[workspace]
members = ["src-tauri"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.95"

[workspace.dependencies]
chrono = { version = "0.4.45", default-features = false, features = ["std", "clock", "serde"] }
chrono-tz = { version = "0.10.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2.0.19"
uuid = { version = "1.24.0", features = ["v7", "serde"] }

[profile.release]
lto = true
strip = true
```

- [ ] **Step 2: Create `src-tauri/Cargo.toml`**

```toml
[package]
name = "self-planning"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
name = "self_planning_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2.11.5", features = [] }
serde = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 3: Create `src-tauri/build.rs` and `src-tauri/tauri.conf.json`**

`build.rs`:

```rust
fn main() {
    tauri_build::build();
}
```

`tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Self-Planning",
  "version": "0.1.0",
  "identifier": "com.gamecoderstudios.self-planning",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Self-Planning",
        "width": 1100,
        "height": 760,
        "minWidth": 900,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"]
  }
}
```

Generate the icon set once with `npx tauri icon` (it accepts a 1024×1024 source PNG; if none
exists, run `npx tauri icon` with no argument to emit the Tauri default set) and commit
`src-tauri/icons/`.

- [ ] **Step 4: Write the failing Rust test**

`src-tauri/src/private/commands.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_the_cargo_manifest() {
        assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
    }
}
```

- [ ] **Step 5: Run the test to verify it fails**

```bash
cargo test --workspace
```

Expected: FAIL — `cannot find function 'app_version' in this scope`.

- [ ] **Step 6: Write the minimal implementation**

Prepend to `src-tauri/src/private/commands.rs`:

```rust
/// Proves the IPC bridge works end to end. Plan 0004 replaces this module's
/// contents with the real application commands.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

`src-tauri/src/private/mod.rs`:

```rust
pub mod commands;
```

`src-tauri/src/lib.rs` — the public interface: wiring only, no logic.

```rust
mod private;

use private::commands::app_version;

/// Builds and runs the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .expect("error while running the Self-Planning application");
}
```

`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    self_planning_lib::run();
}
```

- [ ] **Step 7: Run the test to verify it passes**

```bash
cargo test --workspace
```

Expected: PASS — 1 test.

- [ ] **Step 8: Verify the app actually launches**

```bash
npm run tauri dev
```

Expected: a dark window titled "Self-Planning" showing the heading `Daily Plan`. Close it.
If the window is blank, check that `src/styles/tokens.css` exists (Task 1 Step 7).

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock src-tauri
git commit -m "feat: add Cargo workspace and Tauri 2 shell with app_version command"
```

---

### Task 3: Resolved design tokens with a contrast guard

**Files:**
- Modify: `src/styles/tokens.css`, `DESIGN.md`
- Test: `tests/tokens.test.ts`

**Interfaces:**
- Consumes: `src/styles/tokens.css` created empty in Task 1.
- Produces: the CSS custom properties listed in
  [0001 → Design tokens](../0001-self-planning-app.md#design-tokens-resolved). Plan 0007 styles every
  component exclusively from these; no component may contain a raw hex value.

- [ ] **Step 1: Write the failing contrast test**

`tests/tokens.test.ts`:

```ts
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
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
npx vitest run tests/tokens.test.ts
```

Expected: FAIL — `token --color-ink is not defined in tokens.css`.

- [ ] **Step 3: Write `src/styles/tokens.css`**

```css
/*
 * Focused Night palette — see DESIGN.md and docs/plans/0001-self-planning-app.md.
 * Contrast ratios are enforced by tests/tokens.test.ts. Do not change a colour
 * without re-running that test.
 */
:root {
  /* Surfaces */
  --color-base: #14161a;
  --color-lift: #1d2026;
  --color-raised: #252932;
  --color-hairline: #2e323b;

  /* Text */
  --color-ink: #e8eaed;
  --color-ink-muted: #a3aab6;

  /* Accent — the One Accent Rule: gold covers <=10% of any screen */
  --color-gold: #d4a94a;
  --color-gold-deep: #b88f35;

  /* Semantic state — restrained tint shifts, never punitive floods */
  --color-overdue: #e0a183;
  --color-done: #8fb89c;

  /* Type — one family, fixed rem scale at ratio 1.125 */
  --font-sans: 'Inter', ui-sans-serif, system-ui, 'Segoe UI', Roboto, sans-serif;
  --text-label: 0.75rem;
  --text-body: 0.875rem;
  --text-title: 1rem;
  --text-headline: 1.25rem;
  --text-display: 1.75rem;

  /* Space — 4px base */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-6: 1.5rem;
  --space-8: 2rem;

  /* Shape — cards top out at 16px; pills are for tags and compact buttons only */
  --radius-card: 12px;
  --radius-control: 8px;
  --radius-pill: 999px;

  /* Depth — flat at rest, one hover shadow, one focus elevation. No Material ladder. */
  --shadow-hover: 0 2px 8px rgb(0 0 0 / 0.35);
  --focus-ring: 0 0 0 2px var(--color-base), 0 0 0 4px var(--color-gold);

  /* Motion — 150-250ms ease-out for state changes only */
  --duration-fast: 150ms;
  --duration-state: 250ms;
  --ease-out: cubic-bezier(0.2, 0, 0, 1);
}

@media (prefers-reduced-motion: reduce) {
  :root {
    --duration-fast: 1ms;
    --duration-state: 1ms;
  }
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--color-base);
  color: var(--color-ink);
  font-family: var(--font-sans);
  font-size: var(--text-body);
  font-variant-numeric: tabular-nums;
}

:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
npx vitest run tests/tokens.test.ts
```

Expected: PASS — 13 tests.

- [ ] **Step 5: Replace the placeholders in `DESIGN.md`**

Edit `DESIGN.md` sections 2–4: replace every `[to be resolved during implementation]` with the
resolved value and its measured contrast ratio, replace the font placeholders in section 3 with
`Inter` plus the fallback stack and the fixed rem sizes, and replace section 4's shadow
placeholder with `--shadow-hover` and `--focus-ring`. Delete the `<!-- SEED: ... -->` comment on
line 1. Add a line under the section 2 heading: `Source of truth: src/styles/tokens.css.`

- [ ] **Step 6: Commit**

```bash
git add src/styles/tokens.css tests/tokens.test.ts DESIGN.md
git commit -m "feat: resolve design tokens and guard contrast with tests"
```

---

### Task 4: Architecture guards

**Files:**
- Create: `tests/architecture.test.ts`
- Test: same file (it is the test)

**Interfaces:**
- Consumes: the workspace layout from Task 2.
- Produces: automated enforcement of three Global Constraints so downstream plans cannot violate
  them silently. These tests will fail with "directory not found" style errors only if the
  workspace layout changes — they tolerate crates that do not exist yet.

- [ ] **Step 1: Write the guard tests**

```ts
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
```

- [ ] **Step 2: Run the tests to verify they pass on the current tree**

```bash
npx vitest run tests/architecture.test.ts
```

Expected: PASS. If `globSync` is unavailable on the installed Node version, replace the import
with `import { globSync } from 'node:fs'` → `import { globSync } from 'glob'` and add `glob` to
devDependencies. Node 24 ships `fs.globSync`, so this should not be needed.

- [ ] **Step 3: Prove a guard actually catches a violation**

Temporarily add `use chrono::Local;` to `src-tauri/src/private/commands.rs`, re-run the test,
confirm the "never uses the device local time zone" case FAILS and names that file, then revert
the edit and confirm it passes again.

- [ ] **Step 4: Commit**

```bash
git add tests/architecture.test.ts
git commit -m "test: enforce time, crate boundary, and no-hard-delete guards"
```

---

### Task 5: The `check` gate, lint, and fallow

**Files:**
- Create: `eslint.config.js`, `.prettierrc.json`, `fallow.json`
- Modify: `package.json` (scripts)

**Interfaces:**
- Consumes: everything above.
- Produces: `npm run check` — the single command every later task in every later plan runs as its
  definition of done.

- [ ] **Step 1: Create `eslint.config.js`**

```js
import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';

export default [
  { ignores: ['dist/', 'target/', 'node_modules/', 'coverage/'] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.svelte'],
    languageOptions: { parserOptions: { parser: ts.parser } },
  },
  {
    rules: {
      // GUIDELINES.md: no function takes more than 3 parameters.
      'max-params': ['error', 3],
      // GUIDELINES.md: no source file exceeds 200 lines.
      'max-lines': ['error', { max: 200, skipBlankLines: true, skipComments: true }],
      // GUIDELINES.md: no function exceeds 30 lines.
      'max-lines-per-function': ['error', { max: 30, skipBlankLines: true, skipComments: true }],
    },
  },
];
```

Add `@eslint/js` to devDependencies if `npm run lint` reports it missing.

- [ ] **Step 2: Create `.prettierrc.json`**

```json
{
  "singleQuote": true,
  "printWidth": 100,
  "plugins": ["prettier-plugin-svelte"],
  "overrides": [{ "files": "*.svelte", "options": { "parser": "svelte" } }]
}
```

- [ ] **Step 3: Add the aggregate scripts to `package.json`**

```json
"check": "npm run check:rust && npm run typecheck && npm run lint && npm run test",
"check:rust": "cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace"
```

- [ ] **Step 4: Run the full gate**

```bash
npm run check
```

Expected: PASS. Fix any `max-lines-per-function` violation by splitting — do not raise the limit.
If `cargo fmt --check` fails, run `cargo fmt --all` and re-run.

- [ ] **Step 5: Configure fallow**

```bash
fallow init
```

Then edit the generated `fallow.json` so `src/lib/api`, `src/lib/domain`, `src/lib/ui`, and each
directory under `src/lib/surfaces` are declared boundaries whose only public entry is `index.ts`,
and add `src/main.ts` plus `tests/**` as entry points so they are not reported as dead code.
Verify:

```bash
fallow audit
```

Expected: no findings.

- [ ] **Step 6: Commit**

```bash
git add eslint.config.js .prettierrc.json fallow.json package.json package-lock.json
git commit -m "chore: add npm run check gate, lint config, and fallow boundaries"
```

---

### Task 6: The `api` deep module and shell documentation

**Files:**
- Create: `src/lib/api/index.ts`, `src/lib/api/Private/bridge.ts`,
  `docs/architecture/app-shell.md`
- Modify: `docs/architecture/README.md`, `docs/live/current-status.md`
- Test: `src/lib/api/index.test.ts`

**Interfaces:**
- Consumes: the `app_version` command from Task 2.
- Produces: `appVersion(): Promise<string>` exported from `src/lib/api`. Every later frontend plan
  adds its functions to this same module and never imports `@tauri-apps/api` elsewhere — enforced
  by Task 4.

- [ ] **Step 1: Write the failing test**

`src/lib/api/index.test.ts`:

```ts
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
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
npx vitest run src/lib/api
```

Expected: FAIL — cannot resolve `./index`.

- [ ] **Step 3: Write the implementation**

`src/lib/api/Private/bridge.ts` — the only file in the frontend allowed to import Tauri IPC:

```ts
import { invoke } from '@tauri-apps/api/core';

/** Calls a Tauri command. The single crossing point between the UI and the Rust core. */
export function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return args === undefined ? invoke<T>(command) : invoke<T>(command, args);
}
```

`src/lib/api/index.ts` — the deep module's public interface:

```ts
import { call } from './Private/bridge';

export function appVersion(): Promise<string> {
  return call<string>('app_version');
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
npx vitest run src/lib/api
```

Expected: PASS.

- [ ] **Step 5: Write `docs/architecture/app-shell.md`**

Keep it under 60 lines. It must state: the crate graph and its one-way direction; that `src-tauri`
and `launcher` may depend on `planning-app` only; that `lib.rs` is a crate's public interface and
`src/private/` is its implementation; that `src/lib/api` is the sole IPC crossing point; that
`npm run check` is the gate; and that `tests/architecture.test.ts` enforces all of the above.
Include the file table from this plan's File structure section.

- [ ] **Step 6: Register the doc and correct the status file**

Add to the table in `docs/architecture/README.md`:

```markdown
| [app-shell.md](app-shell.md) | Tauri + Svelte shell, crate graph, quality gates | Enforced by `tests/architecture.test.ts` |
```

Rewrite `docs/live/current-status.md`: move the app shell from "Blocked before implementation" to
"Implemented", delete the stale claim about a missing Phase 1 scaffold, and point at
`docs/plans/0001-self-planning-app.md` as the roadmap index.

- [ ] **Step 7: Run the full gate and commit**

```bash
npm run check && fallow audit
```

```bash
git add src/lib/api docs
git commit -m "feat: add api deep module and document the app shell"
```

---

## Task 7: Verify the plan's own acceptance

- [ ] `npm run tauri dev` opens a dark window reading "Daily Plan" in Inter at 1.75rem. *(compile/launch succeeded earlier; Inter/1.75rem not confirmed in CI-like env)*
- [x] `npm run check` passes from a clean clone after `npm install`.
- [x] `fallow audit` reports no findings.
- [x] Deliberately breaking any one guard in `tests/architecture.test.ts` fails `npm run check`.
- [x] `DESIGN.md` contains no `[to be resolved during implementation]` and no SEED comment.
- [x] `docs/live/current-status.md` no longer claims a missing scaffold.

**Next:** [0003-storage-and-settings.md](../0003-storage-and-settings.md).
