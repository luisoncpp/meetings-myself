# UI surfaces

Svelte 5 surfaces for Daily Plan, Library, and Weekly Review. Each surface owns a store class (`.svelte.ts`), renders through deep-module components, and talks to Rust only via `src/lib/api`.

## Two windows, three surfaces

| Surface | Window | How it is reached |
|---------|--------|-------------------|
| **Daily Plan** | Main (`index.html`) | Default — `mainView === 'daily-plan'` in `AppShell` |
| **Library** | Main | `Navigation` switches `mainView` to `'library'` |
| **Weekly Review** | Separate Tauri window | `?surface=weekly-review` on load, or `openWeeklyReviewWindow()` |

`App.svelte` calls `currentSurface(window.location.search)` once at boot. The pure helper in `src/lib/shell/Private/surface.ts` maps `?surface=weekly-review` → `'weekly-review'`; anything else → `'main'`. On `'main'`, `AppShell` toggles Daily Plan vs Library locally; Weekly Review is never embedded in the main window.

## Store-class pattern

Surface state lives in classes (`DailyPlanStore`, `LibraryStore`, `WeeklyReviewStore`, `SetupStore`) in `*.svelte.ts` files — not hook bundles. Each surface is one coherent state machine; hooks would scatter it across effects and obscure the mutation → reload contract. Classes also match project preference in `docs/GUIDELINES.md` (classes over hooks).

`$state` runes inside these classes require the `.svelte.ts` extension — see [svelte-5-runes-need-the-svelte-ts-extension.md](../lessons-learned/svelte-5-runes-need-the-svelte-ts-extension.md).

## Frontend layout (`src/lib/`)

Deep modules: folders with `index.ts` are public interfaces; `Private/` holds implementation. Only the parent module's `index.ts` may import across a `Private/` boundary.

```
src/lib/
├── api/           ← sole IPC crossing (index.ts + Private/bridge.ts)
├── domain/        ← TypeScript mirror of Rust read models
├── shell/         ← AppShell, health gate, navigation, surface routing
├── surfaces/      ← daily-plan, library, weekly-review, setup
└── ui/            ← shared primitives (see below)
```

Nothing under `src/` except `api/Private/bridge.ts` may import `@tauri-apps/api`.

## Mutations and optimism

| Action | Strategy | Why |
|--------|----------|-----|
| Plan reorder | **Optimistic** — local order updates immediately; snaps back on IPC failure | Drag/keyboard reorder must feel instant |
| Complete, check-in, select, archive, reflection, … | **Pessimistic** — `await` mutation, then `load()` | Projections (`archived`, `overdue`, `state`) come from the server; guessing would lie |

`DailyPlanStore.reorder` is the only optimistic path. Every other mutation goes through `#change`: call API, then reload the authoritative view.

## Health gate

`AppShell` calls `storeHealth()` on mount. Until the first response, no surface renders. Then:

1. `setupIncomplete` → `Setup` surface (pick sync folder / home zone).
2. Any other non-`ready` status → `HealthBanner` blocks all surfaces.
3. `ready` → route to Weekly Review or main (Daily Plan / Library).

Writes are refused server-side when health is not `ready`; the gate keeps the UI from offering actions that would fail.

## Surface map

| Module | Store | Primary API calls |
|--------|-------|-------------------|
| `surfaces/daily-plan` | `DailyPlanStore` | `todayView`, `taskPool`, `reorderPlan`, `completeTask`, `recordCheckIn`, … |
| `surfaces/library` | `LibraryStore` | `library`, `archiveEntity`, `createTask`, `link`, … |
| `surfaces/weekly-review` | `WeeklyReviewStore` | `openCurrentReview`, `saveReflection`, `weeklySummary`, … |
| `surfaces/setup` | `SetupStore` | `chooseSyncFolder`, `setHomeZone`, `pickSyncFolder` |

Shared UI primitives live in `src/lib/ui`; surface-specific rows and panels stay inside each surface's `Private/`.

**Layout:** `SurfaceLayout` — content column with `max-width: min(var(--content-max-width), 100%)` and horizontal padding; used by Daily Plan, Library, Weekly Review, and Setup.

**Forms:** `Field`, `Input`, `Select`, `Textarea` — labelled controls with token-backed borders and backgrounds.

**Other:** `Button`, `Card`, `ListRow`, `OrderableList`, `CheckInControl`, `StateFlag`, `InsetPanel`.

## Accessibility

Verification for Plan 0007 Task 9. Automated checks ran in CI/dev; visual and OS-level checks need a human pass on Windows with `npm run tauri dev`.

### Verified via tests

- **Keyboard — plan reorder:** `OrderableList` reorders with Alt+Arrow, moves focus with arrow keys, and announces position via a live region (`src/lib/ui/Private/OrderableList.test.ts`).
- **Keyboard — habit check-in:** `CheckInControl` is reachable by Tab, operable with arrow keys, and groups options under the habit label (`src/lib/ui/Private/CheckInControl.test.ts`).
- **State not by colour alone:** `StateFlag` renders visible text for every kind (Archived, Overdue, Unpinned, Completed) (`src/lib/ui/Private/StateFlag.test.ts`).
- **Reduced-motion plumbing:** `tokens.css` sets `--duration-fast` and `--duration-state` to `1ms` under `@media (prefers-reduced-motion: reduce)`; components use those tokens instead of hard-coded millisecond transitions (guarded by `tests/design-rules.test.ts`).
- **Contrast pairs:** token foreground/background pairs are checked for WCAG 2.1 AA in `tests/tokens.test.ts`.
- **Store flows exercised in unit tests:** task completion toggle, plan reorder, quick-add, library archive/restore, weekly reflection autosave — behaviour covered at store/component test level; not full end-to-end keyboard walks.

### Pending human confirmation (Windows + Tauri)

Run the desktop app and confirm interactively:

- Every core flow is completable with keyboard alone: complete a task, reorder the plan, record a check-in, quick-add a task, archive and restore from the Library, type and save a reflection.
- Focus ring (`--focus-ring` in `tokens.css`) is visible on every interactive element on each surface background.
- Enabling **Settings → Accessibility → Visual effects → Animation effects** off (reduce motion) removes perceptible UI transitions.
- At **200%** display or browser zoom, no content is clipped or overlapped on Daily Plan, Library, or Weekly Review.
- Spot-check that status flags remain understandable without relying on colour alone in real layouts (tests cover `StateFlag` text; composite rows should be eyeballed).
