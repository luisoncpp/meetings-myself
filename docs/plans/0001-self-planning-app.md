# Self-Planning App — Implementation Roadmap

> **For agentic workers:** This file is an **index**, not an executable plan. Each linked
> plan is executable on its own. Use the `executing-plans` skill (or
> subagent-driven development) on **one plan file at a time**, in the order below.

**Goal:** Build a local-first Tauri 2 desktop app for Daily Plans and Weekly Reviews, storing
synchronized planning data in an embedded SurrealDB/RocksDB directory inside a Google Drive
folder, plus one editable Markdown Weekly Report per Calendar Week.

**Why a plan set:** The spec spans six subsystems that can each be built, tested, and reviewed
independently. One 3000-line plan would not fit a fresh implementer's working memory. Each plan
below produces working, testable software on its own and states exactly what it consumes from
earlier plans.

---

## Preconditions (verified 2026-08-06)

| Fact | Value |
|------|-------|
| Source files in repo | **None.** Repo is docs-only. The Phase 1 scaffold referenced by `docs/live/current-status.md` does not exist and must be created by plan 0002. |
| Node | v24.14.0 |
| npm | 11.15.0 |
| cargo / rustc | 1.95.0 |
| fallow | 3.14.0 (`fallow audit` is part of the definition of done — see `docs/WORKFLOW.md`) |
| Git branch | `master`, single commit `e042927` |
| LLVM / libclang | **Not installed.** Required by `bindgen` for RocksDB. Plan 0003 Task 0 installs it. |
| NASM | **Not installed.** Required by `aws-lc-sys` on MSVC. Plan 0003 Task 0 works around it. |

`docs/live/current-status.md` must be corrected by plan 0002 — it currently claims a scaffold
that is not present.

**Verified build blockers.** A compile probe of `surrealdb 3.2.4` with `kv-rocksdb` on this machine
failed twice before succeeding past both stages: first with `NASM command not found` from
`aws-lc-sys` (reached via `surrealdb-core` → `jsonwebtoken` 10 → `aws-lc-rs`), then with
`Unable to find libclang` from `bindgen` building `surrealdb-librocksdb-sys`. Neither is avoidable
through feature selection on `surrealdb`. **Plan 0003 Task 0 must be completed before any other
work in plan 0003**, and the first RocksDB build takes 10–20 minutes.

---

## Plan order

| # | Plan | Delivers | Depends on |
|---|------|----------|------------|
| 0002 | [App shell & design tokens](done/0002-app-shell.md) | Done — Cargo workspace, Tauri 2 + Svelte 5 + Vite scaffold, both test harnesses, verified WCAG-AA token set, `check` script | — |
| 0003 | [Storage, settings & sync safety](done/0003-storage-and-settings.md) | Done — `planning-core` + `planning-store` + `planning-app`, embedded SurrealDB/RocksDB, device settings, home zone, `StoreHealth` gating | 0002 |
| 0004 | [Planning domain & Library API](done/0004-planning-domain.md) | `planning-core` + `planning-app` crates: Values, Goals, Habits, Tasks, Associations, archive-only lifecycle, overdue projection | 0003 |
| 0005 | [Daily Plans, habits & recurrence](done/0005-daily-plan-and-habits.md) | Done — Daily Plan, Weekly Focus, pinned-habit inclusion, Habit Check-ins, idempotent Recurring Task materialization | 0004 |
| 0006 | [Weekly Reviews & report files](done/0006-weekly-review-and-reports.md) | Done — `planning-reports`, weekly Markdown reports with preserved reflection, regenerated summaries, Weekly Review API | 0005 |
| 0007 | [UI surfaces](done/0007-ui-surfaces.md) | Daily Plan window (home), Library surface, separate Weekly Review window | 0005 (0006 for the review surface) |
| 0008 | [Daily Plan Launcher](0008-launcher.md) | Separate `planning-launcher` binary, 7:00 AM home-time attempt, retry window, missed-prompt record | 0005 |

Plans 0006 and 0007 may run in parallel after 0005 if two workers are available. Everything else
is strictly sequential.

---

## Target repository layout

Locked in by plan 0002. Later plans add files inside it and never restructure it.

```
Cargo.toml                    # workspace root
crates/
  planning-core/              # pure domain: types + rules, no IO
  planning-store/             # SurrealDB/RocksDB persistence + device settings
  planning-app/               # application API (use cases) — the ONLY surface Tauri/launcher call
  planning-reports/           # weekly report Markdown files
src-tauri/                    # Tauri app binary; registers commands, holds no logic
launcher/                     # planning-launcher binary
src/                          # Svelte 5 frontend
  lib/api/                    # thin invoke() wrapper (deep module)
  lib/domain/                 # TS mirror of read-model types
  lib/ui/                     # primitives
  lib/surfaces/               # daily-plan/, library/, weekly-review/ (deep modules)
  styles/tokens.css
docs/
```

### Deep-module rule mapping

`docs/GUIDELINES.md` defines deep modules for TypeScript. The Rust equivalent used throughout
this plan set:

- **A crate is a deep module.** Its `src/lib.rs` is the public interface and contains only
  `mod` declarations and `pub use` re-exports — no logic.
- **Implementation lives in `src/private/`.** Nothing outside the crate may name a `private::`
  path; `lib.rs` re-exports the narrow surface.
- **Crate dependency direction is one-way:**
  `planning-core` ← `planning-store` ← `planning-app` → `planning-reports`
  `src-tauri` and `launcher` depend on `planning-app` **only**. They must never depend on
  `planning-store`, `planning-core`, or `surrealdb` directly. Plan 0002 adds a test that fails
  if they do.

---

## Global constraints

Every task in every plan implicitly includes these. They are copied verbatim from
`docs/GUIDELINES.md`, `docs/adr/`, and `CONTEXT.md`.

### Code style (`docs/GUIDELINES.md`)

- No function takes more than **3 parameters**. Past 3, pass a struct/object.
- No source file exceeds **200 lines**. No function exceeds **30 lines**.
- Hardcoded `true`/`false` at a call site must be written `/*argumentName=*/true`.
- A hardcoded value after a callback must use the same named-parameter comment form.
- Prefer return-early (`return` / `continue`) over nested blocks.
- Prefer classes over hooks in the frontend; if a hook is needed, it is a thin adapter over a class.
- Do not create helper functions solely to group unrelated assignments.

### Domain invariants (`docs/adr/0002`, `CONTEXT.md`)

- **Nothing is ever hard-deleted.** No `DELETE` statement exists anywhere in the codebase except
  in test teardown. Archiving is the only removal, and it is reversible.
- **Forward-only.** Archiving, unpinning, and Habit Cadence changes never rewrite an existing
  Daily Plan or Weekly Focus. Affected entries stay in place, marked, and still completable.
- **Recurring Task rules are factories.** A materialized occurrence is an ordinary Task,
  unaffected by later edits to the rule. Archiving a rule stops future materialization only.
- **Associations never cascade.** Archiving one side leaves the other untouched and preserves the
  link, dormant until restored.
- **Outcomes stay correctable.** Completed Tasks can be reopened; any past Habit Check-in can be
  changed. Completing a Task is never gated on being in a Daily Plan.
- **Weekly Report summaries are regenerated from current data**, never frozen. Only typed
  reflection is preserved verbatim.

### Time (`docs/adr/0001`)

- Every date, week, deadline, and recurrence calculation uses the **synchronized home time zone**,
  never the device time zone. No code calls `Local::now()` or `chrono::Local` — plan 0002 adds a
  test that greps for it and fails the build.
- All "current time" flows through the `Clock` trait so tests can pin it. No production code calls
  `Utc::now()` outside `SystemClock`.

### Synchronization (`docs/adr/0001`)

- One active writer. The app refuses to write when `StoreHealth` is not `Ready`.
- Device-specific settings (launch time, retry window, Synchronization Folder path, device id)
  are stored **outside** the Synchronization Folder and are never synced.

### Definition of done for every task

1. `npm run check` passes (see plan 0002 — runs fmt, clippy, cargo test, svelte-check, eslint, vitest).
2. `fallow audit` reports no new findings.
3. Documentation updated per `docs/UPDATE.md` — architecture doc for new boundaries, flow doc for
   new end-to-end sequences, lessons-learned for surprises.
4. Committed with a conventional-commit message.

---

## Design tokens (resolved)

`DESIGN.md` left colors and typography as placeholders. They are resolved here; plan 0002 writes
them into `src/styles/tokens.css` and updates `DESIGN.md` to match. Contrast ratios were computed
against the WCAG 2.1 relative-luminance formula and are recorded in plan 0002 as test fixtures.

| Token | Value | Role |
|-------|-------|------|
| `--color-base` | `#14161A` | App background |
| `--color-lift` | `#1D2026` | Cards, list rows, panels |
| `--color-raised` | `#252932` | Hover / pressed surface |
| `--color-hairline` | `#2E323B` | Dividers only (non-text) |
| `--color-ink` | `#E8EAED` | Headings, primary text — 15.03:1 on base |
| `--color-ink-muted` | `#A3AAB6` | Metadata, labels — 7.75:1 on base |
| `--color-gold` | `#D4A94A` | Primary action, selection, focus ring — 8.26:1 on base |
| `--color-gold-deep` | `#B88F35` | Primary hover/pressed — 6.05:1 on base |
| `--color-overdue` | `#E0A183` | Overdue marker — 7.46:1 on lift |
| `--color-done` | `#8FB89C` | Done check-in marker — 7.39:1 on lift |

Every text token clears the 4.5:1 AA threshold. `--color-hairline` is decorative and carries no
information, so it is exempt from 1.4.11; nothing may use it for text or as the sole indicator of
a state.

**Type:** one family — `Inter` with a system fallback stack. Fixed rem scale, ratio 1.125:
`--text-label 0.75rem`, `--text-body 0.875rem`, `--text-title 1rem`, `--text-headline 1.25rem`,
`--text-display 1.75rem`. No fluid `clamp()` on UI chrome.

**Named rules that plan 0007 must honor:** gold on ≤10% of any screen; flat at rest, depth only on
interaction; backdrop blur only on transient overlays; no border-plus-wide-shadow pairs; card
radius ≤16px; archived/overdue/skipped shown honestly, never hidden.

---

## Acceptance criteria for the whole app

These are the product-level gates. Each is claimed by a specific plan; the claiming plan contains
the test that proves it.

| # | Criterion | Proven by |
|---|-----------|-----------|
| A1 | A Task created on one device appears on another after Drive syncs and the first device stopped editing | 0003 |
| A2 | A Daily Plan can select, order, remove, and complete Tasks without duplicating them | 0005 |
| A3 | Pinned Habits appear only on their cadence days and record one of three check-in outcomes | 0005 |
| A4 | A Weekly Review shows the prior report, produces exactly one editable report file for its week, and creates a Weekly Focus for the coming week | 0006 |
| A5 | Recurring Tasks never duplicate when the app is reopened | 0005 |
| A6 | Archiving a Task or Habit already in a Daily Plan leaves the entry in place, marked and still completable, and unselectable for future plans | 0004 + 0005 |
| A7 | The launcher never opens the app from unavailable or unsafe synchronized data | 0008 |
| A8 | The Weekly Review has no exclusive powers — every action it offers is also in the Library | 0007 |

---

## Open decisions recorded here (not deferred)

1. **The launcher records its missed prompt device-locally, not in the synced database.** ADR 0001
   requires the launcher to reach synchronized data read-only, and a missed prompt is a
   device-specific fact. Plan 0008 writes it to the device settings file; the app reads that file
   at startup and surfaces it. Plan 0008 adds this as an amendment note to ADR 0001.
2. **Recurring Task materialization runs on app open and on home-date rollover**, not on a timer.
   Idempotency comes from a `(rule_id, occurrence_date)` uniqueness constraint, not from
   `last_materialized` bookkeeping alone. Plan 0005 owns this.
3. **Daily Plans store ids only.** Archived/unpinned marking is projected at read time by resolving
   each id against current entity state. This is what makes forward-only propagation free rather
   than a migration. Plans 0004 and 0005 own this.
4. **Three surfaces, two windows.** Daily Plan (home) and Library live in the `main` window with
   in-app navigation; Weekly Review opens in a separate `weekly-review` Tauri window. Plan 0007
   owns this.
