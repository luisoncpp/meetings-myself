# UI surfaces

Svelte 5 surfaces for Daily Plan, Library, and Weekly Review. Each surface owns a store class (`.svelte.ts`), renders through deep-module components, and talks to Rust only via `src/lib/api`.

## One window, three sections

`AppShell` renders a side rail (`Navigation.svelte`, `MainView = 'today' | 'library' | 'week'`) and one section at a time in the main window: **Today** (`DailyPlan`), **Library**, **Week** (`WeeklyReview`). There is no second Tauri window, no `?surface=` routing, and no window lifecycle code (plan 0009 removed them; see [tauri-hidden-windows-keep-process-alive.md](../lessons-learned/tauri-hidden-windows-keep-process-alive.md), now historical).

Sections are **unmounted on switch** (`{#if}` in `AppShell`), so each one reloads on entry. That is the cross-section freshness contract: a focus change in Week shows in Library and Today with no event bus. Leaving Week runs `WeeklyReviewStore.destroy()`, which saves a pending reflection.

## Store-class pattern

Surface state lives in classes (`DailyPlanStore`, `LibraryStore`, `WeeklyReviewStore`, `SetupStore`) in `*.svelte.ts` files — not hook bundles. Each surface is one coherent state machine; hooks would scatter it across effects and obscure the mutation → reload contract. Classes also match project preference in `docs/GUIDELINES.md` (classes over hooks).

`$state` runes inside these classes require the `.svelte.ts` extension — see [svelte-5-runes-need-the-svelte-ts-extension.md](../lessons-learned/svelte-5-runes-need-the-svelte-ts-extension.md).

## Frontend layout (`src/lib/`)

Deep modules: folders with `index.ts` are public interfaces; `Private/` holds implementation. Only the parent module's `index.ts` may import across a `Private/` boundary.

```
src/lib/
├── api/           ← sole IPC crossing (index.ts + Private/bridge.ts)
├── domain/        ← TypeScript mirror of Rust read models
├── i18n/          ← LocaleStore, catalogs (en/es), t(), localizeError
├── shell/         ← AppShell, health gate, navigation, surface routing
├── surfaces/      ← daily-plan, library, weekly-review, setup
└── ui/            ← shared primitives (see below)
```

Nothing under `src/` except `api/Private/bridge.ts` may import `@tauri-apps/api`.

## Internationalization (en + es)

All app-authored UI copy, domain labels, health/setup messages, and IPC errors are localized through `src/lib/i18n`:

- `LocaleStore` loads `ui_language` from device settings on boot (`App.svelte` gates rendering until ready).
- `t(key, params?)` resolves keys from `Private/catalogs/en.json` / `es.json`.
- `localizeError` maps stable JSON error codes from Rust IPC to catalog strings.
- Language can be changed on **Setup** and in **Navigation** (`LanguageSelect` in `ui/`). Preference is per-device (`DeviceSettings.ui_language`), not synced.
- Plan dates use `formatPlanDate(iso, bcp47(locale))`; Weekly Report summary headings and the Reflection starter are generated in Rust from the same language setting.

User-authored content (task titles, reflections, etc.) is never translated.

## Mutations and optimism

| Action | Strategy | Why |
|--------|----------|-----|
| Plan reorder | **Optimistic** — local order updates immediately; snaps back on IPC failure | Drag/keyboard reorder must feel instant |
| Complete, check-in, select, archive, reflection, … | **Pessimistic** — `await` mutation, then `load()` | Projections (`archived`, `overdue`, `state`) come from the server; guessing would lie |

`DailyPlanStore.reorder` is the only optimistic path. Every other mutation goes through `#change`: call API, then reload the authoritative view.

## Health gate

`AppShell` calls `storeHealth()` on mount. Until the first response, no surface renders. Then:

1. `setupIncomplete` → `Setup` surface (pick sync folder / home zone). Choosing a folder that cannot be opened leaves setup (`onready`) so the health banner can show `unreadable`.
2. Any other non-`ready` status → `HealthBanner` blocks all surfaces. `folderMissing` offers **Try again** (`reconnect_store`) and **Choose a different folder**. Other retryable faults only offer **Try again**.
3. `ready` → side rail + the current section.

**Try again** (`folderMissing`, `lockedByAnotherDevice`, `unreadable`) calls `reconnect()`, which re-runs database open — not `storeHealth()`, which only reads the last snapshot.

Writes are refused server-side when health is not `ready`; the gate keeps the UI from offering actions that would fail.

## Surface map

| Module | Store | Primary API calls |
|--------|-------|-------------------|
| `surfaces/daily-plan` | `DailyPlanStore` | `todayView`, `yesterdayView`, `taskPool`, `weeklyFocus(plan.week)`, `reorderPlan`, `completeTask`, `recordCheckIn`, … |
| `surfaces/library` | `LibraryStore` | `library(true)`, `recurringTasks`, `weeklyFocus` (this + next week), entity edits, `link`, … |
| `surfaces/weekly-review` | `WeeklyReviewStore` | `openCurrentReview`, `saveReflection`, `library(false)`, `weeklyFocus(next)`, `createTask` + `addToFocus`, … |
| `surfaces/setup` | `SetupStore` | `chooseSyncFolder`, `setHomeZone`, `pickSyncFolder` |

## Rows, menu, and focus marks (shared)

- **`TaskLine`** (`ui`) — every task row in every section: one line, round check (`role="checkbox"`), title (a button when the row opens details), `meta` snippet, focus mark, `trailing` snippet, `⋯` button. `TaskChips` shows only overdue / high importance / high urgency; low and unclassified live in the detail panel.
- **Focus marks** — gold dot + `--color-gold-wash` row = this week's focus; gold ring = next week's (Library only). Text for screen readers comes from the mark.
- **Task menu** — `TaskActions` + `TaskMenuController` + `TaskMenu` in `planning-actions`; each store owns one `TaskActions` wired to its reload. Items and IPC: [right-click-task-actions.md](../flows/right-click-task-actions.md).
- **Done = archived for one-offs** — the backend archives a one-off on completion. UIs read `PlanTaskView.completed` / `TaskView.completedOn`, never `state === 'completed'` (archived wins in `state`).

## Today

Top to bottom: date; **Yesterday** bar (collapsed, shows "n open · m habits unchecked"; expands in place with the same rows and check-ins, completions stamped with yesterday); habit strip (`CheckInControl compact` — ✓ ↷ ✕ symbols, full outcome names as accessible labels); two columns — **Today** (`OrderableList` of rows + `InlineAdd`) and **Up next** (pool: focus first, `←` adds to today).

## Library

`TabStrip`: Tasks · Habits · Goals · Values · Recurring · Archive. `LibraryStore` loads everything once with archived included; tabs filter. Each tab has `+ New …` (`CreateEntity`). Clicking a row opens `DetailPanel` on the right (`LibraryDetail` → `TaskDetail` / `HabitDetail` / `GoalDetail` / `ValueDetail`); rows never hold form controls.

- **Task detail** — title (`rename_task`), importance, urgency, deadline, one-off, this/next week focus toggles, links, done/reopen, archive.
- **Habit detail** — title (`rename_habit`), **Show in daily plan** switch (was the pin icon; `pinned` in the model), strength, cadence, links, archive. Unpinned habits show a "Not daily" note on their row.
- **Archive tab** — tasks grouped **Done** (newest `completedOn` first, date shown) / **Not done**; then archived habits, goals, values and recurring rules. Each row restores; task rows keep the check and menu.
- **Recurring tab** — `RecurringTaskSection`; rules are not `EntityKind` rows. Creating a rule stores only the `recurring_task` record; the next `open_today` materializes. See [creating-a-recurring-task.md](../flows/creating-a-recurring-task.md).
- **Links** — `DetailLinks` = `AssociationTags` + `LinkModal` (target kinds Goal → Habit → Value → Task; archived entities are never candidates). See [linking-entities-in-library.md](../flows/linking-entities-in-library.md).

## Week

Header (week label + `WeekNav`), then the **focus board** (`FocusBoard`): *Focus for <next week>* (gold-wash column, `×` removes, `InlineAdd` creates a task **and** adds it to that focus via `createTaskInFocus`) beside *Everything else* (`☆` adds). Below: `PlanningActionBar` (New habit, New goal, Mark achieved — tasks are created on the board), reflection editor, then `ReviewHistory` — this week's summary and the previous report as one-line `<details>` disclosures, and the report path.

Focus edits here target **next week** (`focusWeek = nextWeek(view.week)`); Today uses this week's focus. Library shows both, which is why it has two marks.

## Primitives (`src/lib/ui`)

**Layout:** `SurfaceLayout` (content column, `--content-max-width`), `DetailPanel` (sticky right panel), `TabStrip` (tablist, arrow keys).
**Rows/menus:** `TaskLine`, `TaskChips`, `ContextMenu` (`MenuItem.keepOpen` swaps content via `children`), `InlineAdd`, `OrderableList`, `ListRow` (recurring rows), `CheckInControl`, `StateFlag`.
**Forms:** `Field`, `Input`, `Select`, `MarkdownEditor` (Quill Next; Markdown in/out via `marked` / `getSemanticHTML` + `turndown`), `Button`, `Card`, `LanguageSelect`.

## Accessibility

Verification for Plan 0007 Task 9. Automated checks ran in CI/dev; visual and OS-level checks need a human pass on Windows with `npm run tauri dev`.

### Verified via tests

- **Keyboard — task menu:** right-click, Menu key or the row's `⋯` button opens it; arrows move, Esc/Tab close, focus returns to the row (`DailyPlan.menu.test.ts`, `task-menu-items.test.ts`).
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
