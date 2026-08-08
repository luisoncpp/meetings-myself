# UI Surfaces — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](0001-self-planning-app.md), `PRODUCT.md`, and `DESIGN.md` first.
> Requires [0005-daily-plan-and-habits.md](done/0005-daily-plan-and-habits.md); the Weekly Review surface
> (Task 8) additionally requires [0006-weekly-review-and-reports.md](done/0006-weekly-review-and-reports.md).

**Goal:** Build the Daily Plan home, the Library, and a separate Weekly Review window on top of the
existing application API — calm, keyboard-operable, WCAG 2.1 AA, and honest about archived,
overdue, and corrected state.

**Architecture:** Two Tauri windows, three surfaces. `main` holds the Daily Plan (home) and the
Library with in-app navigation; `weekly-review` is a separate window loading the same bundle with
`?surface=weekly-review`. Per `docs/GUIDELINES.md`, each surface's state lives in a **class** with
Svelte 5 `$state` fields in a `.svelte.ts` file — not in a pile of hooks or module-level stores.
Components read the class and call its methods; they hold no fetch logic.

**Tech Stack:** Svelte 5.56 (runes), Vitest 4.1 + `@testing-library/svelte`, the tokens from plan
0002. No component library, no CSS framework, no drag-and-drop library.

---

## Global constraints

See [0001-self-planning-app.md](0001-self-planning-app.md#global-constraints). Plus, from
`PRODUCT.md` and `DESIGN.md` — these are requirements, not suggestions:

- **Daily Plan is home.** The app opens on it. Library and Weekly Review are reachable without
  competing for attention on that surface.
- **The One Accent Rule.** Gold appears on ≤10% of any screen: primary action, current selection,
  focus ring, and meaningful progress only. Never a background, never a decorative fill.
- **The Real State Rule.** Overdue, archived, skipped, and corrected entries render in place with
  honest labels. Never hidden, never a red storm, never a green flood.
- **No gamification.** No streaks, badges, confetti, celebration animation, percentages, scores, or
  "you're on fire" copy. A check-in is three plain choices.
- **No SaaS dashboard.** No widget grids, hero metrics, KPI cards, or purple gradients.
- **Flat until touched.** Depth on hover/focus only. Backdrop blur only on transient overlays. Never
  a 1px border and a wide soft shadow on the same element. Card radius ≤16px.
- **Accessibility.** Full keyboard operability for every core flow, visible focus states, and a
  `prefers-reduced-motion` alternative for any motion beyond essential feedback. Reordering must
  work from the keyboard — pointer-only drag and drop fails AA.
- **Styling comes from tokens.** `tests/architecture.test.ts` fails the build on a raw hex value in
  a `.svelte` file.
- **Only `src/lib/api/Private/bridge.ts` imports `@tauri-apps/api`.** Also enforced by that test.

---

## File structure

Each directory with an `index.ts` is a deep module; `Private/` holds its implementation.

| File | Responsibility |
|------|----------------|
| `src/lib/ui/index.ts` | Primitives barrel |
| `src/lib/ui/Private/Button.svelte` | Primary / secondary / quiet button |
| `src/lib/ui/Private/Card.svelte` | Solid surface, flat at rest |
| `src/lib/ui/Private/ListRow.svelte` | Dense calm row with slots |
| `src/lib/ui/Private/StateFlag.svelte` | Archived / overdue / unpinned marker |
| `src/lib/ui/Private/CheckInControl.svelte` | Done / Skipped / Not completed |
| `src/lib/ui/Private/OrderableList.svelte` | Pointer + keyboard reordering |
| `src/lib/shell/index.ts` | `AppShell.svelte`, surface routing |
| `src/lib/shell/Private/HealthBanner.svelte` | `StoreHealth` states |
| `src/lib/shell/Private/Navigation.svelte` | Daily Plan / Library / Weekly Review |
| `src/lib/surfaces/setup/` | First-run folder + time zone |
| `src/lib/surfaces/daily-plan/` | Home surface + `DailyPlanStore.svelte.ts` |
| `src/lib/surfaces/library/` | Library surface + `LibraryStore.svelte.ts` |
| `src/lib/surfaces/weekly-review/` | Review surface + `WeeklyReviewStore.svelte.ts` |
| `src-tauri/tauri.conf.json` | *(modify)* second window |
| `docs/architecture/ui-surfaces.md` | New architecture doc |
| `docs/flows/completing-a-task-from-the-daily-plan.md` | New flow doc |

---

### Task 1: UI primitives

**Files:**
- Create: `src/lib/ui/index.ts` and the five `Private/*.svelte` files above (`OrderableList` is
  Task 5)
- Test: `src/lib/ui/Private/Button.test.ts`, `CheckInControl.test.ts`, `StateFlag.test.ts`

**Interfaces:**
- Consumes: the tokens from plan 0002.
- Produces, all exported from `src/lib/ui`:

```ts
// Button.svelte props
{ variant?: 'primary' | 'secondary' | 'quiet'; disabled?: boolean;
  type?: 'button' | 'submit'; onclick?: () => void; children: Snippet }

// Card.svelte props      { interactive?: boolean; children: Snippet }
// ListRow.svelte props   { leading?: Snippet; children: Snippet; trailing?: Snippet;
//                          muted?: boolean; onactivate?: () => void }
// StateFlag.svelte props { kind: 'archived' | 'overdue' | 'unpinned' | 'completed' }
// CheckInControl.svelte props
{ value: CheckInOutcome | null; label: string; onchange: (next: CheckInOutcome) => void }
```

`CheckInControl` is a radio group, not three buttons: exactly one outcome applies at a time, and a
radio group gives arrow-key navigation and correct screen-reader semantics for free.

- [ ] **Step 1: Write the failing tests**

`src/lib/ui/Private/CheckInControl.test.ts`:

```ts
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import CheckInControl from './CheckInControl.svelte';

describe('CheckInControl', () => {
  it('offers exactly the three agreed outcomes and no others', () => {
    render(CheckInControl, { value: null, label: 'Writing practice', onchange: vi.fn() });
    const options = screen.getAllByRole('radio');
    expect(options.map((option) => option.getAttribute('aria-label'))).toEqual([
      'Done',
      'Skipped',
      'Not completed',
    ]);
  });

  it('marks the current outcome and reports a change', async () => {
    const onchange = vi.fn();
    render(CheckInControl, { value: 'done', label: 'Writing practice', onchange });

    expect(screen.getByRole('radio', { name: 'Done' })).toBeChecked();
    await userEvent.click(screen.getByRole('radio', { name: 'Skipped' }));
    expect(onchange).toHaveBeenCalledWith('skipped');
  });

  it('is reachable and operable from the keyboard', async () => {
    const onchange = vi.fn();
    render(CheckInControl, { value: 'done', label: 'Writing practice', onchange });

    await userEvent.tab();
    expect(screen.getByRole('radio', { name: 'Done' })).toHaveFocus();
    await userEvent.keyboard('{ArrowRight}');
    expect(onchange).toHaveBeenCalledWith('skipped');
  });

  it('names the habit it belongs to, so the group is unambiguous', () => {
    render(CheckInControl, { value: null, label: 'Writing practice', onchange: vi.fn() });
    expect(screen.getByRole('radiogroup', { name: /Writing practice/ })).toBeInTheDocument();
  });
});
```

Add `@testing-library/user-event` to devDependencies.

`StateFlag.test.ts` asserts each `kind` renders visible text — `Archived`, `Overdue`, `Unpinned`,
`Completed` — and not colour alone. Colour-only state fails WCAG 1.4.1 and hides the truth from
anyone who cannot see it.

- [ ] **Step 2: Run to verify they fail**

```bash
npx vitest run src/lib/ui
```

Expected: FAIL — cannot resolve `./CheckInControl.svelte`.

- [ ] **Step 3: Implement `Button.svelte`**

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'quiet';
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: () => void;
    children: Snippet;
  }

  let {
    variant = 'secondary',
    disabled = false,
    type = 'button',
    onclick,
    children,
  }: Props = $props();
</script>

<button class={variant} {type} {disabled} {onclick}>
  {@render children()}
</button>

<style>
  button {
    padding: var(--space-2) var(--space-4);
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    font: inherit;
    font-size: var(--text-body);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  /* The One Accent Rule: solid gold is rare, and only ever a primary action. */
  .primary {
    background: var(--color-gold);
    color: var(--color-base);
    font-weight: 600;
  }

  .primary:hover:not(:disabled) {
    background: var(--color-gold-deep);
  }

  .secondary {
    background: var(--color-raised);
    color: var(--color-ink);
    border-color: var(--color-hairline);
  }

  .secondary:hover:not(:disabled) {
    background: var(--color-lift);
  }

  .quiet {
    background: none;
    color: var(--color-ink-muted);
  }

  .quiet:hover:not(:disabled) {
    color: var(--color-ink);
  }
</style>
```

- [ ] **Step 4: Implement `CheckInControl.svelte`**

```svelte
<script lang="ts">
  import type { CheckInOutcome } from '../../domain';

  interface Props {
    value: CheckInOutcome | null;
    label: string;
    onchange: (next: CheckInOutcome) => void;
  }

  let { value, label, onchange }: Props = $props();

  // Exactly three outcomes, forever. PRODUCT.md forbids scoring, so there is no
  // fourth state and no partial credit.
  const OUTCOMES: ReadonlyArray<{ value: CheckInOutcome; label: string }> = [
    { value: 'done', label: 'Done' },
    { value: 'skipped', label: 'Skipped' },
    { value: 'notCompleted', label: 'Not completed' },
  ];
</script>

<div role="radiogroup" aria-label="Check-in for {label}" class="group">
  {#each OUTCOMES as outcome (outcome.value)}
    <button
      type="button"
      role="radio"
      aria-label={outcome.label}
      aria-checked={value === outcome.value}
      tabindex={value === outcome.value || (value === null && outcome.value === 'done') ? 0 : -1}
      class:selected={value === outcome.value}
      onclick={/* record this outcome */ () => onchange(outcome.value)}
    >
      {outcome.label}
    </button>
  {/each}
</div>

<style>
  .group {
    display: flex;
    gap: var(--space-1);
  }

  button {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-label);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  /* Selection is the accent's job. No celebration, no animation on check-in. */
  .selected {
    border-color: var(--color-gold);
    color: var(--color-gold);
  }
</style>
```

Arrow-key navigation within the group is the roving-`tabindex` pattern: add an `onkeydown` handler
on the group that moves the selection on `ArrowRight`/`ArrowLeft` and calls `onchange`. Keep it
under 30 lines by extracting `nextOutcome(current, direction)` as a plain function in
`Private/outcome-navigation.ts` with its own unit test.

- [ ] **Step 5: Implement `Card.svelte`, `ListRow.svelte`, `StateFlag.svelte`**

`StateFlag.svelte` renders a small pill with **text**, tinted per kind:
`archived` → `--color-ink-muted`; `overdue` → `--color-overdue`; `unpinned` → `--color-ink-muted`;
`completed` → `--color-done`. Never colour alone.

`Card.svelte` is a solid `--color-lift` surface with `--radius-card` and **no border** — the
Flat-By-Default Rule prohibits pairing a border with a shadow. `interactive` adds
`--shadow-hover` on hover only.

- [ ] **Step 6: Run, write `index.ts`, commit**

```ts
export { default as Button } from './Private/Button.svelte';
export { default as Card } from './Private/Card.svelte';
export { default as CheckInControl } from './Private/CheckInControl.svelte';
export { default as ListRow } from './Private/ListRow.svelte';
export { default as StateFlag } from './Private/StateFlag.svelte';
```

```bash
npx vitest run src/lib/ui
```

Expected: PASS.

```bash
git add src/lib/ui package.json package-lock.json
git commit -m "feat(ui): add calm primitives with accessible check-in control"
```

---

### Task 2: The shell — routing and the health banner

**Files:**
- Create: `src/lib/shell/index.ts`, `Private/AppShell.svelte`, `Private/Navigation.svelte`,
  `Private/HealthBanner.svelte`, `Private/surface.ts`
- Modify: `src/App.svelte`, `src-tauri/tauri.conf.json`
- Test: `src/lib/shell/Private/HealthBanner.test.ts`, `surface.test.ts`

**Interfaces:**
- Consumes: `storeHealth()` from `src/lib/api`.
- Produces:
  - `currentSurface(search: string): 'main' | 'weekly-review'` — reads `?surface=`; pure and
    unit-testable, so the routing decision is not buried in a component.
  - `AppShell.svelte` — renders Setup, Daily Plan, Library, or Weekly Review depending on surface
    and health.
  - A second Tauri window, label `weekly-review`, `visible: false` at startup.

**The health gate is the shell's job:** when `StoreHealth` is not `ready`, no surface renders its
normal content. Showing a stale Daily Plan over unsafe data would be exactly the failure ADR 0001
exists to prevent.

- [ ] **Step 1: Write the failing tests**

`surface.test.ts`:

```ts
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
```

`HealthBanner.test.ts`:

```ts
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import HealthBanner from './HealthBanner.svelte';

describe('HealthBanner', () => {
  it('renders nothing when the store is ready', () => {
    const { container } = render(HealthBanner, { health: { status: 'ready' } });
    expect(container.textContent?.trim()).toBe('');
  });

  it('names the other device when the data is locked', () => {
    render(HealthBanner, {
      health: { status: 'lockedByAnotherDevice', deviceName: 'laptop', since: '2026-08-07T09:00:00Z' },
    });
    expect(screen.getByRole('alert')).toHaveTextContent(/laptop/);
  });

  it('explains a sync conflict without offering a destructive fix', () => {
    render(HealthBanner, { health: { status: 'syncConflict', artifacts: ['CURRENT (1)'] } });
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent(/CURRENT \(1\)/);
    expect(alert.textContent).not.toMatch(/delete/i);
  });

  it('tells the user what to do when the folder is missing', () => {
    render(HealthBanner, { health: { status: 'folderMissing', path: 'D:/Drive/planning' } });
    expect(screen.getByRole('alert')).toHaveTextContent(/D:\/Drive\/planning/);
  });
});
```

The "no destructive fix" assertion is deliberate: the app must never offer to delete a Drive
conflict artifact. Resolving it is the user's call, made outside the app.

- [ ] **Step 2: Run to verify they fail, then implement**

```bash
npx vitest run src/lib/shell
```

`surface.ts`:

```ts
export type Surface = 'main' | 'weekly-review';

/** Pure so the routing rule is testable without a window. */
export function currentSurface(search: string): Surface {
  return new URLSearchParams(search).get('surface') === 'weekly-review'
    ? 'weekly-review'
    : 'main';
}
```

`HealthBanner.svelte` renders a `role="alert"` region per non-ready status, using
`--color-overdue` for the marker and plain sentences for the explanation. It offers a "Try again"
button (calling a `onretry` prop) for `folderMissing` and `lockedByAnotherDevice`, and for
`syncConflict` it lists the artifact paths and explains that the files must be resolved in Google
Drive first. No status offers a delete.

- [ ] **Step 3: Add the second window to `tauri.conf.json`**

```json
{
  "label": "weekly-review",
  "title": "Weekly Review",
  "url": "index.html?surface=weekly-review",
  "width": 900,
  "height": 760,
  "visible": false
}
```

Add a Rust command in `src-tauri/src/private/window_commands.rs`:

```rust
/// Shows the Weekly Review window, creating nothing — the window is declared in
/// tauri.conf.json so all Tauri APIs stay on the Rust side of the boundary.
#[tauri::command]
pub fn open_weekly_review_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let window = app
        .get_webview_window("weekly-review")
        .ok_or("the weekly-review window is not configured")?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}
```

and `export function openWeeklyReviewWindow(): Promise<void>` in `src/lib/api/index.ts`.

- [ ] **Step 4: Rewrite `src/App.svelte` to delegate to `AppShell`**

`App.svelte` becomes three lines: import `AppShell`, compute the surface from
`window.location.search`, render. Delete the placeholder heading from plan 0002 and update
`src/App.test.ts` to assert the shell renders rather than a bare heading.

- [ ] **Step 5: Run, commit**

```bash
npm run check
```

```bash
git add src src-tauri
git commit -m "feat(shell): add surface routing, health gating, and the review window"
```

---

### Task 3: The setup surface

**Files:**
- Create: `src/lib/surfaces/setup/index.ts`, `Private/Setup.svelte`,
  `Private/SetupStore.svelte.ts`
- Modify: `src-tauri/src/private/window_commands.rs` (folder picker)
- Test: `src/lib/surfaces/setup/Private/Setup.test.ts`

**Interfaces:**
- Consumes: `storeHealth`, `chooseSyncFolder`, `setHomeZone` from plan 0003.
- Produces: a two-step first-run flow — pick the Synchronization Folder, then the home time zone.
  Until both are done, `StoreHealth` is `setupIncomplete` and no other surface renders.

The folder picker needs a native dialog. Add `tauri-plugin-dialog` 2.7 to `src-tauri`, register it,
and expose `pick_sync_folder() -> Option<PathBuf>` as a Rust command — again keeping Tauri plugin
APIs out of the frontend.

Time zones: offer `chrono_tz::TZ_VARIANTS` from a Rust command `available_time_zones() -> Vec<String>`,
rendered as a searchable `<datalist>`-backed input. The app suggests nothing — ADR 0001's amendment
says the home zone starts unset on purpose.

- [ ] **Step 1: Write the failing test**

```ts
it('walks from no folder, to no zone, to ready', async () => {
  const chooseSyncFolder = vi.fn().mockResolvedValue({
    status: 'setupIncomplete',
    reason: { kind: 'NoHomeZone' },
  });
  const setHomeZone = vi.fn().mockResolvedValue({ status: 'ready' });
  vi.doMock('../../../api', () => ({
    chooseSyncFolder,
    setHomeZone,
    pickSyncFolder: vi.fn().mockResolvedValue('D:/Drive/self-planning'),
    availableTimeZones: vi.fn().mockResolvedValue(['Europe/Madrid', 'UTC']),
  }));

  const { default: Setup } = await import('./Setup.svelte');
  const onready = vi.fn();
  render(Setup, { health: { status: 'setupIncomplete', reason: { kind: 'NoSyncFolder' } }, onready });

  await userEvent.click(screen.getByRole('button', { name: /choose folder/i }));
  expect(chooseSyncFolder).toHaveBeenCalledWith('D:/Drive/self-planning');

  await userEvent.type(screen.getByLabelText(/home time zone/i), 'Europe/Madrid');
  await userEvent.click(screen.getByRole('button', { name: /finish setup/i }));
  expect(setHomeZone).toHaveBeenCalledWith('Europe/Madrid');
  expect(onready).toHaveBeenCalled();
});

it('explains why the time zone matters instead of just asking for it', () => {
  render(Setup, { health: { status: 'setupIncomplete', reason: { kind: 'NoHomeZone' } }, onready: vi.fn() });
  expect(screen.getByText(/every device/i)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run to verify it fails, implement, run, commit**

`SetupStore.svelte.ts` holds `$state` for the chosen folder, the zone text, and the last error;
`Setup.svelte` renders two `Card`s and calls the store. Copy is calm and explanatory: the folder
step says the folder should be inside Google Drive and that only one device may edit at a time; the
zone step says the choice governs day and week boundaries on *every* device.

```bash
npm run check
```

```bash
git add src src-tauri
git commit -m "feat(setup): add first-run folder and home time zone selection"
```

---

### Task 4: `DailyPlanStore`

**Files:**
- Create: `src/lib/surfaces/daily-plan/Private/DailyPlanStore.svelte.ts`
- Test: `src/lib/surfaces/daily-plan/Private/DailyPlanStore.test.ts`

**Interfaces:**
- Consumes: `todayView`, `taskPool`, `selectIntoPlan`, `removeFromPlan`, `reorderPlan`,
  `quickAddTask`, `recordCheckIn`, `completeTask`, `reopenTask` from `src/lib/api`.
- Produces:

```ts
export class DailyPlanStore {
  readonly plan: DailyPlanView | null;
  readonly pool: TaskPoolView | null;
  readonly loading: boolean;
  readonly error: string | null;

  load(): Promise<void>;
  select(taskId: string): Promise<void>;
  remove(taskId: string): Promise<void>;
  reorder(order: string[]): Promise<void>;
  quickAdd(title: string): Promise<void>;
  checkIn(habitId: string, outcome: CheckInOutcome): Promise<void>;
  toggleCompletion(task: PlanTaskView): Promise<void>;
}
```

**The file must be `.svelte.ts`.** Svelte 5 runes are a compiler feature; `$state` in a plain `.ts`
file silently does nothing reactive. This is the single most common Svelte 5 mistake and the reason
the extension is called out here rather than left to be discovered.

**Optimistic reorder, pessimistic everything else.** Drag-and-drop must feel immediate, so
`reorder` updates local order first and reloads on failure. The rest reload after the call — the
cost is imperceptible against a local database, and it keeps projections (`overdue`, `archived`)
authoritative rather than guessed at in TypeScript.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it, vi, beforeEach } from 'vitest';

const api = {
  todayView: vi.fn(),
  taskPool: vi.fn(),
  selectIntoPlan: vi.fn().mockResolvedValue(undefined),
  removeFromPlan: vi.fn().mockResolvedValue(undefined),
  reorderPlan: vi.fn().mockResolvedValue(undefined),
  quickAddTask: vi.fn().mockResolvedValue({ id: 't9' }),
  recordCheckIn: vi.fn().mockResolvedValue(undefined),
  completeTask: vi.fn().mockResolvedValue(undefined),
  reopenTask: vi.fn().mockResolvedValue(undefined),
};
vi.mock('../../../api', () => api);

function planWith(taskIds: string[]) {
  return {
    date: '2026-08-07',
    week: '2026-W32',
    tasks: taskIds.map((id, position) => ({
      id, title: id, state: 'open', importance: 'unclassified', urgency: 'unclassified',
      deadline: null, overdue: false, archived: false, position,
    })),
    habits: [],
  };
}

describe('DailyPlanStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.todayView.mockResolvedValue(planWith(['a', 'b', 'c']));
    api.taskPool.mockResolvedValue({ focus: [], rest: [] });
  });

  it('loads the plan and the pool together', async () => {
    const { DailyPlanStore } = await import('./DailyPlanStore.svelte');
    const store = new DailyPlanStore();
    await store.load();

    expect(store.plan?.tasks).toHaveLength(3);
    expect(store.pool).toEqual({ focus: [], rest: [] });
    expect(store.loading).toBe(false);
  });

  it('applies a reorder optimistically and sends the new order', async () => {
    const { DailyPlanStore } = await import('./DailyPlanStore.svelte');
    const store = new DailyPlanStore();
    await store.load();

    const pending = store.reorder(['c', 'a', 'b']);
    expect(store.plan?.tasks.map((task) => task.id)).toEqual(['c', 'a', 'b']);
    await pending;
    expect(api.reorderPlan).toHaveBeenCalledWith('2026-08-07', ['c', 'a', 'b']);
  });

  it('restores the server order when a reorder is rejected', async () => {
    const { DailyPlanStore } = await import('./DailyPlanStore.svelte');
    const store = new DailyPlanStore();
    await store.load();
    api.reorderPlan.mockRejectedValueOnce(new Error('the proposed order is not a permutation'));

    await store.reorder(['c', 'a', 'b']);
    expect(store.plan?.tasks.map((task) => task.id)).toEqual(['a', 'b', 'c']);
    expect(store.error).toMatch(/permutation/);
  });

  it('surfaces a failure without wiping the plan already on screen', async () => {
    const { DailyPlanStore } = await import('./DailyPlanStore.svelte');
    const store = new DailyPlanStore();
    await store.load();
    api.selectIntoPlan.mockRejectedValueOnce(new Error('the task is archived'));

    await store.select('z');
    expect(store.error).toMatch(/archived/);
    expect(store.plan?.tasks).toHaveLength(3);
  });

  it('toggles completion in both directions', async () => {
    const { DailyPlanStore } = await import('./DailyPlanStore.svelte');
    const store = new DailyPlanStore();
    await store.load();

    await store.toggleCompletion(store.plan!.tasks[0]!);
    expect(api.completeTask).toHaveBeenCalledWith('a');

    api.todayView.mockResolvedValue({
      ...planWith(['a']),
      tasks: [{ ...planWith(['a']).tasks[0], state: 'completed' }],
    });
    await store.load();
    await store.toggleCompletion(store.plan!.tasks[0]!);
    expect(api.reopenTask).toHaveBeenCalledWith('a');
  });
});
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
npx vitest run src/lib/surfaces/daily-plan
```

```ts
import type { CheckInOutcome, DailyPlanView, PlanTaskView, TaskPoolView } from '../../../domain';
import * as api from '../../../api';

/**
 * State for the Daily Plan surface.
 *
 * A class rather than a bundle of hooks (docs/GUIDELINES.md): the surface has one
 * coherent state machine, and hooks would scatter it across effects.
 */
export class DailyPlanStore {
  #plan = $state<DailyPlanView | null>(null);
  #pool = $state<TaskPoolView | null>(null);
  #loading = $state(false);
  #error = $state<string | null>(null);

  get plan(): DailyPlanView | null {
    return this.#plan;
  }

  get pool(): TaskPoolView | null {
    return this.#pool;
  }

  get loading(): boolean {
    return this.#loading;
  }

  get error(): string | null {
    return this.#error;
  }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [plan, pool] = await Promise.all([api.todayView(), api.taskPool()]);
      this.#plan = plan;
      this.#pool = pool;
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
    } finally {
      this.#loading = false;
    }
  }

  /**
   * Optimistic: dragging must feel immediate. The server order wins on failure,
   * so a rejected permutation snaps back rather than lying.
   */
  async reorder(order: string[]): Promise<void> {
    const plan = this.#plan;
    if (!plan) return;

    const previous = plan.tasks;
    this.#plan = { ...plan, tasks: reindex(order, previous) };
    try {
      await api.reorderPlan(plan.date, order);
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
      this.#plan = { ...plan, tasks: previous };
    }
  }

  async select(taskId: string): Promise<void> {
    await this.#change(/* selectTheTask= */ () => api.selectIntoPlan(this.#date(), taskId));
  }

  async remove(taskId: string): Promise<void> {
    await this.#change(/* removeTheTask= */ () => api.removeFromPlan(this.#date(), taskId));
  }

  async quickAdd(title: string): Promise<void> {
    await this.#change(/* createAndSelect= */ () => api.quickAddTask(title));
  }

  async checkIn(habitId: string, outcome: CheckInOutcome): Promise<void> {
    await this.#change(/* recordTheOutcome= */ () =>
      api.recordCheckIn(habitId, this.#date(), outcome),
    );
  }

  /** Completion is reversible, so this is a toggle, not a one-way action. */
  async toggleCompletion(task: PlanTaskView): Promise<void> {
    const reopening = task.state === 'completed';
    await this.#change(/* toggleTheOutcome= */ () =>
      reopening ? api.reopenTask(task.id) : api.completeTask(task.id),
    );
  }

  /** Runs a mutation, then reloads so every projection stays authoritative. */
  async #change(mutation: () => Promise<unknown>): Promise<void> {
    try {
      await mutation();
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
      return;
    }
    await this.load();
  }

  #date(): string {
    return this.#plan?.date ?? '';
  }
}

function reindex(order: string[], tasks: PlanTaskView[]): PlanTaskView[] {
  const byId = new Map(tasks.map((task) => [task.id, task]));
  return order
    .map((id, position) => {
      const task = byId.get(id);
      return task ? { ...task, position } : null;
    })
    .filter((task): task is PlanTaskView => task !== null);
}

function message(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure);
}
```

Tauri rejects with a plain string, not an `Error`; `message` handles both.

- [ ] **Step 3: Run, commit**

```bash
npx vitest run src/lib/surfaces/daily-plan
```

Expected: PASS — 5 tests.

```bash
git add src/lib/surfaces/daily-plan
git commit -m "feat(daily-plan): add the plan store with optimistic reordering"
```

---

### Task 5: Keyboard-and-pointer reordering

**Files:**
- Create: `src/lib/ui/Private/OrderableList.svelte`, `src/lib/ui/Private/reorder.ts`
- Modify: `src/lib/ui/index.ts`
- Test: `src/lib/ui/Private/reorder.test.ts`, `OrderableList.test.ts`

**Interfaces:**
- Produces:
  - `move(order: readonly string[], MoveRequest { id, direction }): string[]` — pure, in
    `reorder.ts`, where `direction` is `'up' | 'down'`; a move past either end returns the input
    unchanged.
  - `OrderableList.svelte` props: `{ items: T[]; getId: (item: T) => string; onreorder: (order: string[]) => void; children: Snippet<[T]> }`

Pointer drag alone fails WCAG 2.1 AA (2.1.1 Keyboard). Every row is focusable and responds to
`Alt+ArrowUp` / `Alt+ArrowDown`, announcing the change through an `aria-live="polite"` region.
`Alt` is the modifier because bare arrows must keep moving focus between rows.

- [ ] **Step 1: Write the failing pure-function test**

```ts
import { describe, expect, it } from 'vitest';
import { move } from './reorder';

describe('move', () => {
  const order = ['a', 'b', 'c'];

  it('moves an item up and down', () => {
    expect(move(order, { id: 'b', direction: 'up' })).toEqual(['b', 'a', 'c']);
    expect(move(order, { id: 'b', direction: 'down' })).toEqual(['a', 'c', 'b']);
  });

  it('does nothing at the ends', () => {
    expect(move(order, { id: 'a', direction: 'up' })).toEqual(order);
    expect(move(order, { id: 'c', direction: 'down' })).toEqual(order);
  });

  it('ignores an unknown id', () => {
    expect(move(order, { id: 'z', direction: 'up' })).toEqual(order);
  });

  it('never loses or duplicates an entry', () => {
    const moved = move(order, { id: 'c', direction: 'up' });
    expect([...moved].sort()).toEqual([...order].sort());
  });
});
```

- [ ] **Step 2: Run to verify it fails, then implement `reorder.ts`**

```ts
export interface MoveRequest {
  id: string;
  direction: 'up' | 'down';
}

/** Pure and total: an impossible move returns the input unchanged. */
export function move(order: readonly string[], request: MoveRequest): string[] {
  const from = order.indexOf(request.id);
  if (from === -1) return [...order];

  const to = request.direction === 'up' ? from - 1 : from + 1;
  if (to < 0 || to >= order.length) return [...order];

  const next = [...order];
  [next[from], next[to]] = [next[to]!, next[from]!];
  return next;
}
```

- [ ] **Step 3: Write and pass the `OrderableList` tests**

```ts
it('reorders from the keyboard and announces the result', async () => {
  const onreorder = vi.fn();
  render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

  const row = screen.getByRole('option', { name: 'b' });
  row.focus();
  await userEvent.keyboard('{Alt>}{ArrowUp}{/Alt}');

  expect(onreorder).toHaveBeenCalledWith(['b', 'a', 'c']);
  expect(screen.getByRole('status')).toHaveTextContent(/b.*position 1 of 3/i);
});

it('moves focus between rows with bare arrow keys', async () => {
  render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder: vi.fn() });
  screen.getByRole('option', { name: 'a' }).focus();
  await userEvent.keyboard('{ArrowDown}');
  expect(screen.getByRole('option', { name: 'b' })).toHaveFocus();
});

it('reorders by pointer drag', async () => {
  const onreorder = vi.fn();
  render(OrderableListHarness, { items: ['a', 'b', 'c'], onreorder });

  const source = screen.getByRole('option', { name: 'c' });
  const target = screen.getByRole('option', { name: 'a' });
  await fireEvent.dragStart(source);
  await fireEvent.dragOver(target);
  await fireEvent.drop(target);

  expect(onreorder).toHaveBeenCalledWith(['c', 'a', 'b']);
});
```

`OrderableListHarness.svelte` is a tiny test-only wrapper rendering `OrderableList` with a plain
string snippet. Put it beside the test.

Implement `OrderableList.svelte` as `role="listbox"` with `role="option"` rows, roving `tabindex`,
`draggable="true"`, and an `aria-live="polite"` `role="status"` region reporting the moved item's
new position. Respect `prefers-reduced-motion` by disabling the drop transition — the tokens
already collapse the durations, so simply use `--duration-state` rather than a hard-coded value.

- [ ] **Step 4: Run, export, commit**

```bash
npx vitest run src/lib/ui
```

Expected: PASS.

```bash
git add src/lib/ui
git commit -m "feat(ui): add keyboard-and-pointer orderable list"
```

---

### Task 6: The Daily Plan surface

**Files:**
- Create: `src/lib/surfaces/daily-plan/index.ts`, `Private/DailyPlan.svelte`,
  `Private/PlanTaskRow.svelte`, `Private/HabitList.svelte`, `Private/TaskPool.svelte`,
  `Private/QuickAdd.svelte`
- Test: `src/lib/surfaces/daily-plan/Private/DailyPlan.test.ts`

**Interfaces:**
- Consumes: `DailyPlanStore`, `OrderableList`, the primitives.
- Produces the home surface: the date as the only Display-sized text on screen, an ordered task
  list, habit check-ins, a Weekly-Focus-first Task Pool, and quick add.

Layout: a single column of `Card` sections — Today's Tasks, Habits, Task Pool. No widget grid, no
counts-as-hero-metrics, no progress bar. Associations are **not** shown here; the Daily Plan is for
doing, and links belong to the Library and the Weekly Review.

- [ ] **Step 1: Write the failing surface test**

```ts
it('shows the date, the ordered tasks, and the habits due today', async () => {
  api.todayView.mockResolvedValue({
    date: '2026-08-07',
    week: '2026-W32',
    tasks: [
      { id: 't1', title: 'Draft the letter', state: 'open', importance: 'high',
        urgency: 'unclassified', deadline: null, overdue: false, archived: false, position: 0 },
    ],
    habits: [
      { id: 'h1', title: 'Writing practice', cadence: { kind: 'everyDay' },
        archived: false, unpinned: false, outcome: null },
    ],
  });

  render(DailyPlan);
  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent(/7 August 2026/);
  expect(screen.getByText('Draft the letter')).toBeInTheDocument();
  expect(screen.getByRole('radiogroup', { name: /Writing practice/ })).toBeInTheDocument();
});

it('shows archived and overdue entries in place with honest labels', async () => {
  api.todayView.mockResolvedValue({
    date: '2026-08-07',
    week: '2026-W32',
    tasks: [
      { id: 't1', title: 'Old idea', state: 'archived', importance: 'unclassified',
        urgency: 'unclassified', deadline: null, overdue: false, archived: true, position: 0 },
      { id: 't2', title: 'File taxes', state: 'open', importance: 'high',
        urgency: 'high', deadline: '2026-08-06', overdue: true, archived: false, position: 1 },
    ],
    habits: [],
  });

  render(DailyPlan);
  expect(await screen.findByText('Old idea')).toBeInTheDocument();
  expect(screen.getByText('Archived')).toBeInTheDocument();
  expect(screen.getByText('Overdue')).toBeInTheDocument();
});

it('still lets an archived entry be completed', async () => {
  // A6 at the UI layer: the row stays actionable.
  /* ...same archived fixture... */
  render(DailyPlan);
  await userEvent.click(await screen.findByRole('checkbox', { name: /Old idea/ }));
  expect(api.completeTask).toHaveBeenCalledWith('t1');
});

it('quick-adds a task into today', async () => {
  render(DailyPlan);
  await userEvent.type(await screen.findByLabelText(/add a task/i), 'Call the bank{Enter}');
  expect(api.quickAddTask).toHaveBeenCalledWith('Call the bank');
});

it('lists Weekly Focus tasks before the rest of the pool', async () => {
  api.taskPool.mockResolvedValue({
    focus: [{ id: 'f1', title: 'Prepare portfolio', state: 'open', importance: 'unclassified',
              urgency: 'unclassified', deadline: null, overdue: false, archived: false }],
    rest: [{ id: 'r1', title: 'Something else', state: 'open', importance: 'unclassified',
             urgency: 'unclassified', deadline: null, overdue: false, archived: false }],
  });

  render(DailyPlan);
  const pool = await screen.findByRole('region', { name: /task pool/i });
  const titles = within(pool).getAllByRole('button', { name: /add to today/i });
  expect(titles[0]).toHaveAccessibleName(/Prepare portfolio/);
});
```

- [ ] **Step 2: Run to verify they fail, implement, run**

```bash
npx vitest run src/lib/surfaces/daily-plan
```

Keep each component under 200 lines and each function under 30. `DailyPlan.svelte` composes;
`PlanTaskRow.svelte` renders one row (checkbox, title, `StateFlag`s, importance/urgency badges);
`HabitList.svelte` maps habits to `CheckInControl`; `TaskPool.svelte` renders the two groups with a
visible "In this week's focus" subheading; `QuickAdd.svelte` is a labelled input that submits on
Enter and clears.

Date formatting: `new Intl.DateTimeFormat(undefined, { dateStyle: 'long' })` over the
`YYYY-MM-DD` string parsed as a **local** date — never `new Date('2026-08-07')`, which parses as
UTC and can render the previous day west of Greenwich. Put the conversion in
`src/lib/domain/Private/plan-date.ts` with its own test covering a negative-offset zone.

- [ ] **Step 3: Commit**

```bash
git add src/lib/surfaces/daily-plan
git commit -m "feat(daily-plan): add the home surface with honest state and quick add"
```

---

### Task 7: The Library surface

**Files:**
- Create: `src/lib/surfaces/library/index.ts`, `Private/Library.svelte`,
  `Private/LibraryStore.svelte.ts`, `Private/EntitySection.svelte`,
  `Private/AssociationEditor.svelte`, `Private/CreateEntity.svelte`
- Test: `Private/LibraryStore.test.ts`, `Private/Library.test.ts`

**Interfaces:**
- Consumes: the Library API from plan 0004 and the Weekly Focus API from plan 0005.
- Produces the canonical management home: create, edit, archive, and restore every entity kind;
  edit Associations; achieve Goals; adjust any week's Weekly Focus.

Editable fields per kind — the Library is the **only** place several of these can be changed, so a
missing control means a domain capability the user cannot reach:

| Kind | Editable here |
|------|---------------|
| Value | title, archive / restore |
| Goal | title, target date, achieved / not achieved, archive / restore |
| Habit | title, **Habit Strength**, Habit Cadence, pinned, archive / restore |
| Task | title, Importance, Urgency, Deadline, completed / reopened, archive / restore |
| Recurring Task | title, archive / restore (the recurrence rule itself is fixed at creation) |

`Habit Strength` is manual by definition (`CONTEXT.md`) — the app never infers or advances it. It
renders as a four-option select: Reminder-dependent, Cue-triggered, Strengthening, Established. It
is a description, not a score: no ordering badge, no progress bar, no "level up".

**This surface claims acceptance criterion A8**, so it must offer *every* action the Weekly Review
offers: achieving a Goal, creating a Goal, linking associations, and adjusting the coming Weekly
Focus. A test asserts each is present.

- [ ] **Step 1: Write the failing tests**

```ts
it('offers every action the Weekly Review offers', async () => {
  render(Library);
  await screen.findByRole('heading', { name: 'Library' });

  // A8: nothing below is exclusive to the Weekly Review.
  expect(screen.getByRole('button', { name: /new goal/i })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /mark achieved/i })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /link/i })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /weekly focus/i })).toBeInTheDocument();
});

it('hides archived entries until asked, then shows them labelled', async () => {
  api.library.mockImplementation((includeArchived: boolean) =>
    Promise.resolve({
      values: [], goals: [], habits: [],
      tasks: includeArchived
        ? [{ id: 't1', title: 'Old idea', state: 'archived', archived: true,
             importance: 'unclassified', urgency: 'unclassified', deadline: null, overdue: false }]
        : [],
    }),
  );

  render(Library);
  expect(await screen.findByText(/no tasks/i)).toBeInTheDocument();

  await userEvent.click(screen.getByRole('switch', { name: /show archived/i }));
  expect(await screen.findByText('Old idea')).toBeInTheDocument();
  expect(screen.getByText('Archived')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /restore/i })).toBeInTheDocument();
});

it('never offers to delete anything', async () => {
  render(Library);
  await screen.findByRole('heading', { name: 'Library' });
  expect(screen.queryByRole('button', { name: /delete/i })).not.toBeInTheDocument();
  expect(screen.queryByText(/permanently/i)).not.toBeInTheDocument();
});

it('lets Habit Strength be set by hand, without scoring it', async () => {
  api.library.mockResolvedValue({
    values: [], goals: [], tasks: [],
    habits: [{ id: 'h1', title: 'Writing', cadence: { kind: 'everyDay' },
               strength: 'reminderDependent', pinned: true, archived: false }],
  });

  render(Library);
  const strength = await screen.findByLabelText(/habit strength/i);
  expect(strength).toHaveValue('reminderDependent');

  await userEvent.selectOptions(strength, 'strengthening');
  expect(api.setHabitStrength).toHaveBeenCalledWith('h1', 'strengthening');

  // It is a description, not a level.
  expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
  expect(screen.getByRole('region', { name: /habits/i }).textContent).not.toMatch(/level|\d+\s*\/\s*4/i);
});

it('exposes every editable field the domain offers for a task', async () => {
  api.library.mockResolvedValue({
    values: [], goals: [], habits: [],
    tasks: [{ id: 't1', title: 'File taxes', state: 'open', importance: 'unclassified',
              urgency: 'unclassified', deadline: null, overdue: false, archived: false }],
  });

  render(Library);
  await screen.findByText('File taxes');
  for (const label of [/importance/i, /urgency/i, /deadline/i]) {
    expect(screen.getByLabelText(label)).toBeInTheDocument();
  }
});

it('requires a cadence before a habit can be created', async () => {
  render(Library);
  await userEvent.click(await screen.findByRole('button', { name: /new habit/i }));
  await userEvent.type(screen.getByLabelText(/habit name/i), 'Meditation');

  expect(screen.getByRole('button', { name: /create habit/i })).toBeDisabled();
  await userEvent.click(screen.getByRole('checkbox', { name: 'Monday' }));
  expect(screen.getByRole('button', { name: /create habit/i })).toBeEnabled();
});
```

The "never offers to delete" test is the UI-level guard for ADR 0002. It is cheap and it will catch
a well-meaning future contributor.

- [ ] **Step 2: Run to verify they fail, implement, run, commit**

`LibraryStore.svelte.ts` follows `DailyPlanStore`'s shape: `$state` for the view, the
`includeArchived` toggle, and the error; one `#change` helper; a reload after every mutation.

`Library.svelte` renders four `EntitySection`s (Values, Goals, Habits, Tasks), each a `Card` with a
heading, a list of `ListRow`s, and a create control. Archived rows render `muted` with a
`StateFlag` and a Restore button. `AssociationEditor.svelte` shows an entity's active links and
offers linking to the kinds `CONTEXT.md` permits — nothing else appears in its picker, so an
unsupported pair is unreachable rather than merely rejected.

```bash
npm run check
```

```bash
git add src/lib/surfaces/library
git commit -m "feat(library): add the canonical management surface with no delete path"
```

---

### Task 8: The Weekly Review surface

**Files:**
- Create: `src/lib/surfaces/weekly-review/index.ts`, `Private/WeeklyReview.svelte`,
  `Private/WeeklyReviewStore.svelte.ts`, `Private/PreviousReport.svelte`,
  `Private/ReflectionEditor.svelte`
- Test: `Private/WeeklyReviewStore.test.ts`, `Private/WeeklyReview.test.ts`

**Requires plan 0006.**

**Interfaces:**
- Consumes: `openCurrentReview`, `openWeeklyReview`, `saveReflection`, plus the same Library and
  focus API the Library surface uses.
- Produces the review window: the week label as the Display heading, the prior week's report, the
  regenerated summary, a reflection editor, and contextual actions — achieve a Goal, create a Goal,
  link an Association, adjust next week's focus.

**Reflection must not be lost.** The editor autosaves on blur and on a 2-second debounce, and shows
a plain "Saved" / "Saving…" indicator. A test covers unmount-with-pending-changes.

- [ ] **Step 1: Write the failing tests**

```ts
it('shows the prior report, the regenerated summary, and next week Focus', async () => {
  api.openCurrentReview.mockResolvedValue({
    week: '2026-W32',
    summary: { week: '2026-W32', completed: ['Prepare portfolio'], stillOpen: 2,
               overdue: [], habits: [{ title: 'Writing', done: 4, skipped: 1, notCompleted: 2 }],
               goalsAchieved: [] },
    reflection: '## Reflection\n\nA quiet week.\n',
    previousReport: '## Week in review\n\nLast week happened.\n',
    nextWeekFocus: [],
    reportPath: 'D:/Drive/weekly-reports/2026-W32-weekly-report.md',
  });

  render(WeeklyReview);
  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('2026-W32');
  expect(screen.getByText(/Last week happened/)).toBeInTheDocument();
  expect(screen.getByText('Prepare portfolio')).toBeInTheDocument();
  expect(screen.getByDisplayValue(/A quiet week/)).toBeInTheDocument();
});

it('shows habit counts without scoring them', async () => {
  render(WeeklyReview);
  await screen.findByRole('heading', { level: 1 });
  const habits = screen.getByRole('table', { name: /habits/i });
  expect(habits).toHaveTextContent('Writing');
  expect(habits.textContent).not.toMatch(/%|streak|score/i);
});

it('autosaves the reflection and says so', async () => {
  vi.useFakeTimers();
  render(WeeklyReview);
  const editor = await screen.findByLabelText(/reflection/i);

  await userEvent.type(editor, ' More thoughts.');
  expect(screen.getByRole('status')).toHaveTextContent(/unsaved|saving/i);

  await vi.advanceTimersByTimeAsync(2000);
  expect(api.saveReflection).toHaveBeenCalledWith('2026-W32', expect.stringContaining('More thoughts.'));
  expect(screen.getByRole('status')).toHaveTextContent(/saved/i);
});

it('saves pending reflection changes before the window closes', async () => {
  const { unmount } = render(WeeklyReview);
  await userEvent.type(await screen.findByLabelText(/reflection/i), ' Late edit.');
  unmount();
  expect(api.saveReflection).toHaveBeenCalled();
});

it('links to the report file so the user knows it is theirs to edit', async () => {
  render(WeeklyReview);
  expect(await screen.findByText(/2026-W32-weekly-report\.md/)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run to verify they fail, implement, run, commit**

`WeeklyReviewStore.svelte.ts` owns the view, the draft reflection, a `saveState` of
`'saved' | 'unsaved' | 'saving'`, and a debounce timer it clears on `destroy()`.
`WeeklyReview.svelte` calls `store.destroy()` from `$effect`'s cleanup so the unmount test passes.

Week navigation: previous/next buttons calling `openWeeklyReview(week)`, so a past review can be
reopened. The window shows a quiet note when viewing a week other than the current one — honest,
not a warning.

```bash
npm run check
```

```bash
git add src/lib/surfaces/weekly-review
git commit -m "feat(weekly-review): add the review window with autosaved reflection"
```

---

### Task 9: Accessibility and design-rule verification

**Files:**
- Create: `tests/design-rules.test.ts`
- Modify: `tests/architecture.test.ts` if a new guard is needed

- [ ] **Step 1: Write automated design-rule guards**

```ts
import { readFileSync, globSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const components = globSync('src/**/*.svelte').map((path) => ({
  path,
  text: readFileSync(path, 'utf8'),
}));

describe('DESIGN.md rules', () => {
  it('pairs no 1px border with a wide shadow on the same rule', () => {
    const offenders = components
      .filter(({ text }) => /border:\s*1px[\s\S]{0,200}?box-shadow:\s*[^;]*\d{2,}px/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });

  it('keeps card radius at or below 16px', () => {
    const offenders = components
      .flatMap(({ path, text }) =>
        [...text.matchAll(/border-radius:\s*(\d+)px/g)].map((match) => ({ path, px: +match[1]! })),
      )
      .filter(({ px }) => px > 16 && px < 100) // >=100px is a deliberate pill
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });

  it('uses no gradient text and no backdrop blur outside overlays', () => {
    const offenders = components
      .filter(({ path }) => !path.includes('Overlay') && !path.includes('Popover'))
      .filter(({ text }) => /backdrop-filter|background-clip:\s*text/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual([]);
  });

  it('contains no gamification language anywhere in the UI', () => {
    const banned = /streak|badge|confetti|leaderboard|you're on fire|keep it up/i;
    expect(components.filter(({ text }) => banned.test(text)).map(({ path }) => path)).toEqual([]);
  });

  it('guards motion behind prefers-reduced-motion via the duration tokens', () => {
    const offenders = components
      .filter(({ text }) => /transition:[^;]*\b\d{3,}ms/.test(text))
      .map(({ path }) => path);
    expect(offenders).toEqual(
      [],
      'use var(--duration-fast) or var(--duration-state) so reduced motion applies',
    );
  });
});
```

- [ ] **Step 2: Verify accessibility by hand and record the result**

Run the app (`npm run tauri dev`) and confirm, writing the outcome into
`docs/architecture/ui-surfaces.md`:

- Every core flow is completable with the keyboard alone: complete a task, reorder the plan, record
  a check-in, quick-add a task, archive and restore from the Library, type and save a reflection.
- The focus ring is visible on every interactive element against every surface it sits on.
- Enabling "reduce motion" in Windows settings removes transitions.
- At 200% browser zoom no content is clipped or overlapped.
- Nothing conveys state by colour alone — every flag has text.

- [ ] **Step 3: Run the full gate and commit**

```bash
npm run check && fallow audit
```

```bash
git add tests docs
git commit -m "test: guard the design rules and record the accessibility pass"
```

---

### Task 10: Documentation

**Files:**
- Create: `docs/architecture/ui-surfaces.md`,
  `docs/flows/completing-a-task-from-the-daily-plan.md`,
  `docs/lessons-learned/svelte-5-runes-need-the-svelte-ts-extension.md`
- Modify: the three README index tables, `docs/live/current-status.md`, `DESIGN.md`

- [ ] **Step 1: Write `docs/architecture/ui-surfaces.md`** (target 90 lines)

Cover: two windows and three surfaces, and how `currentSurface` routes them; the store-class pattern
and why classes rather than hooks; that `src/lib/api` is the only IPC crossing; the deep-module
layout of `src/lib/`; which mutations are optimistic (reorder only) and why; the health gate; and
the accessibility results from Task 9.

- [ ] **Step 2: Write the flow doc**

`completing-a-task-from-the-daily-plan.md`: Trigger (user ticks a row) →
`DailyPlanStore.toggleCompletion` → `complete_task` command → `PlanningApp::complete_task` →
home-zone date → store → reload → reprojection. Include the reverse (reopening) and the
archived-entry case, which is the surprising one.

- [ ] **Step 3: Write the lessons-learned entry**

Topic: Svelte 5 runes are a compile-time transform, so `$state` only works in `.svelte` files and
`.svelte.ts` modules. A store class in a plain `.ts` file compiles, runs, and silently never
updates the UI — no error, no warning, just a screen that does not change. Note the second trap:
`vi.mock` of the API module must be hoisted above a dynamic `import()` of the store, or the real
module is captured.

- [ ] **Step 4: Refresh `DESIGN.md` section 5**

Replace the "[To be documented when UI components exist]" placeholder with the real component
inventory: `Button`, `Card`, `ListRow`, `StateFlag`, `CheckInControl`, `OrderableList` — each with
its variants and the rule it enforces.

- [ ] **Step 5: Register everything, update `current-status.md`, commit**

```bash
git add docs DESIGN.md
git commit -m "docs: document the UI surfaces, completion flow, and the runes lesson"
```

---

## Task 11: Verify the plan's own acceptance

- [ ] `npm run check` and `fallow audit` both pass.
- [ ] The app opens on the Daily Plan; Library and Weekly Review are reachable without dominating it.
- [ ] **A8:** every action the Weekly Review offers is present in the Library — proven by the Task 7 test.
- [ ] Archived and overdue entries appear in place with text labels, and archived rows are still
      completable.
- [ ] Every core flow is completable with the keyboard alone, including reordering.
- [ ] No `.svelte` file contains a raw hex colour, gamification language, or an unguarded transition.
- [ ] The Library offers no delete action anywhere.
- [ ] Every editable field in the Task 7 table is reachable from the Library — in particular Habit
      Strength, which has no other home in the UI.
- [ ] With `StoreHealth` not `ready`, no surface renders its normal content.

**Next:** [0008-launcher.md](0008-launcher.md).
