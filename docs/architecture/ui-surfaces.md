# UI surfaces

Architecture notes for the three application surfaces (Daily Plan, Library, Weekly Review). Task 10 expands this document with surface inventory, data flow, and component maps.

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
