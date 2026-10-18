# UI Redesign — Implementation Plan

> **Done 2026-10-17.** Requires [0007-ui-surfaces.md](0007-ui-surfaces.md). Keeps the Focused Night palette
> (`src/styles/tokens.css`) and the "Quiet Ledger" tone in `DESIGN.md`.

**Goal:** Replace walls of text and window/scroll juggling with one window, one-line rows, a detail
panel for editing, and a right-click menu that works on every task anywhere.

**Kept on purpose:** desaturated palette, calm tone, Today / Library / Week as separate sections,
creating entities from the Week section.

## Decisions (agreed 2026-10-17)

| Topic | Decision |
|-------|----------|
| Weekly Review | Becomes the third section of the main window. Separate window and `?surface=weekly-review` route are removed. |
| Done one-off task | Completing archives it immediately. Today keeps showing it struck-through until the day ends. Reopening restores it. |
| Library editing | Rows are one line. Clicking a row opens a right-hand detail panel holding every edit control. |
| Rollout | One redesign pass. |

## Behaviour changes

1. **Auto-archive on completion** — `complete_task` / `complete_task_on` archive one-off Tasks in the
   same write; `reopen_task` restores them. Lives in `planning-app` (not UI) so every caller gets it.
   Recurring occurrences are one-off Tasks too, so a done occurrence is archived; the rule keeps
   producing new ones. Non-one-off (ongoing) Tasks are unaffected.
2. **`TaskView.completedOn: string | null`** — `state` collapses archived over completed, so the
   Archive view needs the completion axis separately.
3. **`create_task` id on the TS side** — the command already returned the `Task`; `api.createTask` is now typed `{ id }` so the Week section can add it to next week's focus. `PlanTaskView.completed` was added alongside `completedOn` for the same reason as 2.
4. **`rename_task`** — needed by the right-click "Rename" (habits and recurring rules already rename).
5. **Focus is visible everywhere** — a gold dot + faint gold wash marks focus tasks on Today, Library
   and Week rows. New token `--color-gold-wash` (contrast test updated).
6. **One window = always fresh** — sections remount on switch and reload, so focus edits in Week show
   in Library and Today with no extra sync code.
7. **Pins removed from rows** — "Show in daily plan" becomes a labelled switch in the habit detail
   panel; rows show a muted "not daily" chip only when unpinned.

## Layout

```
┌──────────┬───────────────────────────────────────────────┐
│ Today    │  <section content>                            │
│ Library  │                                               │
│ Week     │                                               │
│ ──────── │                                               │
│ Folder   │                                               │
│ Language │                                               │
└──────────┴───────────────────────────────────────────────┘
```

- **Today:** date + "Yesterday: n open — Catch up" chip (opens an inline panel, no scrolling);
  habit strip of check-in chips; two columns: *Today* (orderable, inline add) and *Up next* (focus
  first, one-click add to today).
- **Library:** tab strip Tasks / Habits / Goals / Values / Recurring / Archive, a "+ New" button per
  tab, one-line rows with link chips, detail panel on the right. `PlanningActionBar` is removed;
  "Mark achieved" moves to the goal row's menu and detail panel.
- **Archive tab:** archived Tasks grouped *Done* (with date) / *Not done*, plus archived habits,
  goals and values; each row restores. Replaces the global "Show archived" switch.
- **Week:** focus board — *Next week's focus* (gold wash, remove ×, inline "+ New task" that lands in
  focus) beside *Everything else* (star to add). Reflection editor below; previous report and summary
  collapse to one-line disclosures.

## Shared building blocks (`src/lib/ui` + `src/lib/planning-actions`)

| Piece | Role |
|-------|------|
| `SideRail` (shell) | Section switch, folder, language |
| `TaskLine` (ui) | One-line row: check circle, title, focus dot, state chips, `oncontextmenu` |
| `ContextMenu` (ui) | Positioned menu, keyboard (Shift+F10 / Menu key, arrows, Esc), focus return |
| `TaskMenu` (planning-actions) | Done-and-archive / reopen, add-remove focus, add to today, rename, link, archive / restore |
| `DetailPanel` (ui) | Right panel shell; Library rows render their editors inside it |
| `TabStrip` (ui) | Library kinds |

`TaskMenu` actions go through a host interface (like `PlanningActionsHost`) so each section's store
stays the only thing that mutates and reloads.

## Tasks

1. Backend: auto-archive on complete / restore on reopen, `completedOn`, `create_task` returns view,
   `rename_task` + Tauri commands + TS API. Regression tests in `planning-app`.
2. Shell: `SideRail`, Week as in-window section; remove weekly window, its command and
   `tauri.conf.json` entry.
3. UI primitives: `TaskLine`, `ContextMenu`, `DetailPanel`, `TabStrip`, gold-wash token.
4. `TaskMenu` + host binding; wire into Today, Library, Week.
5. Today redesign (habit strip, two columns, yesterday chip/panel).
6. Library redesign (tabs, rows, detail panel, Archive tab, habit pin switch).
7. Week redesign (focus board, auto-focus on create, collapsible summary/previous report).
8. i18n (en + es) for all new copy.
9. Tests, `npm run check`, Rust checks, `fallow audit`; fix new findings.
10. Docs: `ui-surfaces.md`, `planning-domain.md` (auto-archive rule), `DESIGN.md` (rows, menu, panel),
    flows (`completing-a-task…`, `archiving-an-entity`, new `right-click-task-actions.md`),
    mark `tauri-hidden-windows-keep-process-alive.md` historical, `current-status.md`.

## Out of scope

- Renaming goals and values (no IPC yet; can follow the `rename_task` pattern later).
- Drag-and-drop between columns (one-click add stays).
- Launcher (plan 0008).
