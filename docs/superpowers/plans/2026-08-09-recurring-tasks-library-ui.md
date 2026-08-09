# Recurring Tasks Library UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose Recurring Task create/list/archive/restore/rename in the Library UI on top of the existing backend.

**Architecture:** Backend materialization already works. Frontend gap only. Recurring Tasks stay out of association `EntityKind`. Dedicated Library section + store methods.

**Tech Stack:** Svelte 5, Vitest, Tauri IPC, planning-core serde.

## Global Constraints

- Follow `docs/GUIDELINES.md`: deep modules, ≤200-line files, ≤30-line functions, named-parameter comments for bools/callbacks.
- Do **not** commit unless the orchestrator explicitly asks.
- Do **not** change completed-task / Task Pool visibility.
- Recurrence is fixed at creation (read-only on rows).

---

### Task 1: Recurrence Weekly weekday JSON

**Files:**
- Modify: `crates/planning-core/src/private/recurrence.rs`
- Possibly reuse names from: `crates/planning-core/src/private/cadence.rs` (`NAMES` / parse helpers)

**Interfaces:**
- Produces: `Recurrence::Weekly { weekday }` serializes/deserializes as `{"kind":"weekly","weekday":"thu"}` (mon–sun), matching `src/lib/domain/index.ts`.

- [ ] **Step 1: Write failing JSON round-trip tests** in `recurrence.rs` for Daily, Weekdays, Weekly(thu), MonthlyDay(15).
- [ ] **Step 2: Run** `cargo test -p planning-core recurrence -- --nocapture` (unrestricted; empty `CARGO_TARGET_DIR`) — expect Weekly weekday shape fail.
- [ ] **Step 3: Fix serde** so weekday uses `"mon"`…`"sun"` (custom serialize/deserialize or newtype).
- [ ] **Step 4: Re-run tests** — pass.

---

### Task 2: rename_recurring_task IPC + API

**Files:**
- Modify: `src-tauri/src/private/plan_commands.rs`
- Modify: `src-tauri/src/lib.rs` (import + handler list)
- Modify: `src/lib/api/index.ts`
- Modify: `src/lib/api/plan.test.ts`

**Interfaces:**
- Consumes: `PlanningApp::rename_recurring_task`
- Produces: `renameRecurringTask(rule: string, title: string): Promise<void>` calling `rename_recurring_task`

- [ ] **Step 1: Add failing API test** for `renameRecurringTask`.
- [ ] **Step 2: Add Tauri command + register it.**
- [ ] **Step 3: Add `renameRecurringTask` wrapper.**
- [ ] **Step 4: Run** `npm test -- src/lib/api/plan.test.ts` — pass.

---

### Task 3: LibraryStore recurring rules

**Files:**
- Modify: `src/lib/surfaces/library/Private/LibraryStore.svelte.ts`
- Modify: `src/lib/surfaces/library/Private/LibraryStore.test.ts`

**Interfaces:**
- Produces getters/methods:
  - `recurringTasks: RecurringTask[]` (filtered by `includeArchived`)
  - `createRecurringTask(title, recurrence)`
  - `archiveRecurringTask(id)` / `restoreRecurringTask(id)` / `renameRecurringTask(id, title)`
- `load()` also fetches `api.recurringTasks()`.

- [ ] **Step 1: Write failing store tests** (load filters archived; create/archive/restore/rename call API then reload).
- [ ] **Step 2: Implement store fields/methods.**
- [ ] **Step 3: Run store tests** — pass.

---

### Task 4: Create form + section + row

**Files:**
- Create: `src/lib/surfaces/library/Private/CreateRecurrence.svelte`
- Create: `src/lib/surfaces/library/Private/CreateRecurringTask.svelte`
- Create: `src/lib/surfaces/library/Private/RecurringTaskRow.svelte`
- Create: `src/lib/surfaces/library/Private/RecurringTaskSection.svelte`
- Create: `src/lib/surfaces/library/Private/recurrence-label.ts` (human-readable label)
- Modify: `src/lib/surfaces/library/Private/Library.svelte`

**UI rules:**
- New recurring task; submit when title + valid recurrence.
- Row: editable title (when active), read-only recurrence label, Archive/Restore.
- Mount section after Tasks.

- [ ] **Step 1: Implement components** (keep files ≤200 lines).
- [ ] **Step 2: Wire into Library.svelte.**

---

### Task 5: Library surface tests

**Files:**
- Modify: `src/lib/surfaces/library/Private/Library.test.ts` (mock recurring APIs)

- [ ] **Step 1: Tests** for section heading, create flow, show archived, archive/restore.
- [ ] **Step 2: Run** `npm test -- src/lib/surfaces/library` — pass.
- [ ] **Step 3: Run** `npx fallow audit` on touched paths if available.

---

### Task 6: Docs

**Files:**
- Modify: `docs/architecture/ui-surfaces.md`
- Create: `docs/flows/creating-a-recurring-task.md`
- Modify: `docs/flows/README.md`

- [x] **Step 1: Document Library Recurring Tasks capability.**
- [x] **Step 2: Add flow** (create → API → materialize on open_today → Task Pool).
- [x] **Step 3: Index the flow in README.**
