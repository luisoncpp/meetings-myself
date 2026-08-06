# Plan: Self-planning app

## Goal

Build a local-first Tauri desktop app for Daily Plans and Weekly Reviews. It stores shared planning data in an embedded SurrealDB/RocksDB directory synchronized by Google Drive for desktop, and stores one editable Markdown Weekly Report per Calendar Week.

## Preconditions

The workspace currently has planning documentation but no Tauri/Svelte source files, although `docs/live/current-status.md` reports a Phase 1 scaffold. Locate or restore that scaffold before feature work; do not create application features against an assumed structure.

## Delivery sequence

1. **Restore and verify the app shell**
   - Locate or recreate the Tauri 2 + Svelte + TypeScript scaffold.
   - Confirm the build, type-check, Rust checks, and test commands.
   - Establish deep-module boundaries for the Rust core and Svelte feature surfaces.

2. **Build the local data foundation**
   - Create a storage module that opens embedded SurrealDB with RocksDB in the user-selected Synchronization Folder.
   - Separate synchronized planning data from per-device settings such as launcher time, retry window, and local folder path.
   - Store the synchronized home time zone and enforce it for all date, week, deadline, and recurrence calculations.
   - Expose a narrow application API for safe reads and writes; the UI must not access storage directly.

3. **Implement the planning domain**
   - Model Values, Goals, Habits, Tasks, Associations, and their lifecycles.
   - Support high/low/unclassified importance and urgency, date-only deadlines, completion, reversible archiving with no hard delete, and overdue projection.
   - Implement Weekly Focus and reversible Daily Plan task selection.
   - Implement Habit Cadence, persistent pinning, manual one-day addition, and Done/Skipped/Not completed check-ins.
   - Implement duplicate-safe Recurring Task materialization for the agreed daily, weekday, weekly, and monthly patterns; materialized occurrences are ordinary Tasks unaffected by later rule edits.
   - Apply lifecycle changes forward-only: existing Daily Plans and Weekly Focuses keep archived or unpinned entries marked with their new state and still completable; future selections and auto-inclusion follow the new rules.
   - Preserve Associations through archiving: links never cascade to the other side and become active again when the archived side is restored.
   - Create Tasks, Values, and Goals from a title alone (defaults: unclassified Importance/Urgency, no Deadline, optional target date); Habits require a Habit Cadence at creation (default pinned and Reminder-dependent), and Recurring Tasks require a recurrence pattern.
   - Keep recorded outcomes correctable: completed Tasks can be reopened and past Habit Check-ins corrected at any time, and completing a Task is never gated on being in a Daily Plan.

4. **Implement Weekly Reviews and report files**
   - Create and edit one report per Monday–Sunday Calendar Week.
   - Generate automatic task and habit summaries that reflect the latest data while preserving typed reflection sections.
   - Read and write deterministic `weekly-reports/YYYY-Www-weekly-report.md` files with app-owned YAML front matter and externally editable Markdown bodies.
   - Reopen past reviews without creating duplicate reports.

5. **Implement the focused app surfaces**
   - Build a Library surface as the canonical home for creating, editing, and archiving Values, Goals, Habits, Tasks, and their Associations, reachable at any time.
   - Build a Daily Plan window with an ordered, drag-and-drop task list, a Weekly Focus-first Task Pool, and habit check-ins, plus contextual shortcuts such as quick-adding a Task, which also selects it into the day's plan; Associations are displayed read-only and task selection stays manual.
   - Build a separate Weekly Review window for goals, the previous report, task focus, and reflection, with contextual lifecycle actions such as marking a Goal achieved, creating Goals mid-review, and contextual Association linking while reflecting.
   - Give the Weekly Review no exclusive powers: every action it offers, including achieving Goals and adjusting the coming Weekly Focus, is also available in the Library at any time.
   - Keep domain behavior in core modules; use the UI only as an adapter over the public application API.

6. **Implement synchronization-safe operation and the launcher**
   - Detect unavailable or unsafe synchronization data and prevent writes from stale state.
   - Build the separate Daily Plan Launcher around the same read-only API: at 7:00 AM home time it opens the app only when no Daily Plan exists, retries after startup or sync recovery during a configurable morning window, and records a missed prompt when the window ends.
   - Document the one-active-writer workflow and recovery behavior for Drive conflicts or interrupted synchronization.

7. **Verify and document each milestone**
   - Add domain-unit tests before UI integration, then command and component tests for each flow.
   - Run lint, type checks, Rust checks, and the full test suite at every milestone.
   - Add architecture and flow documentation as behavior becomes implemented; move this plan to `docs/plans/done/` only when all stages are complete.

## First implementation slice

Start with storage configuration, home-time-zone handling, and the Task Pool with manual Tasks. This establishes the shared data boundary needed by every later feature without prematurely building UI or automation.

## Acceptance criteria

- A Task created on one device appears on another after Google Drive synchronizes and the first device has stopped editing.
- A Daily Plan can select, order, remove, and complete Tasks without duplicating them.
- Pinned Habits appear only on their due cadence days and record one of the three agreed check-in outcomes.
- A Weekly Review shows the prior report, produces exactly one editable report file for its week, and creates a Weekly Focus for the coming week.
- Recurring Tasks never duplicate when the app is reopened.
- Archiving a Task or Habit that is already in a Daily Plan leaves the entry in place, marked and still completable, and it cannot be selected into future plans.
- The launcher never opens the app from unavailable or unsafe synchronized data.
