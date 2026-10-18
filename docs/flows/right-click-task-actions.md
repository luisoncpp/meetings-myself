# Right-click Task Actions

## Trigger

Right-click (or the Menu key / Shift+F10, or the row's `⋯` button) on any task row in Today, Up next, Yesterday, Library Tasks/Archive, or the Week focus board.

## Entry point

`TaskLine.onmenu(point)` → surface calls `menu.open(taskRef(task) | planTaskRef(task), point)` on its `TaskMenuController` → `<TaskMenu controller actions>` renders `ContextMenu`.

## Pieces

| Piece | Where | Role |
|-------|-------|------|
| `TaskLine` | `ui` | One-line row; fires `onmenu` from `contextmenu` or `⋯` |
| `ContextMenu` | `ui` | Fixed-position menu, scrim click/Esc/Tab closes, arrows move, focus returns to the row |
| `TaskMenuController` | `planning-actions` | `$state` of open task + point; one per surface component |
| `taskMenuItems` | `planning-actions` | Pure item list from `TaskRef` + `TaskActions` |
| `TaskActions` | `planning-actions` | API calls; `run` = the owning store's mutate-then-reload |

Each store owns `readonly actions = new TaskActions({ run, focusTargets, planDate? })`. The menu never mutates state itself; the surface reloads after every action.

## Items (in order)

1. **Done and archive** → `completeTask(id, planDate?)` then `archiveEntity(task)` (the second call matters for ongoing tasks) — or **Reopen** when done.
2. One entry per `FocusTarget`: Today = this week; Library = this week + next week; Week = the week after the reviewed one. Label flips Add/Remove by membership.
3. **Add to today / Remove from today** — only where `planDate` is configured (Today surface).
4. **Rename…** — `keepOpen` item: the menu swaps its items for an inline form (`children` snippet) and calls `renameTask` on Enter.
5. **Archive / Restore**.

## Writes

| Action | IPC |
|--------|-----|
| Done and archive | `complete_task`, `archive_entity` |
| Reopen | `reopen_task` |
| Focus | `add_to_focus` / `remove_from_focus` with the target week |
| Today | `select_into_plan` / `remove_from_plan` |
| Rename | `rename_task` (blank → `BlankTitle`) |
| Archive / Restore | `archive_entity` / `restore_entity` |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| `invalid_snippet_arguments` | A snippet was wrapped in an arrow to pass as `children`; pass the snippet itself |
| Menu item does nothing on the Week board | Review store has no view yet → `focusWeek` is `''` |
| Done from Yesterday stamps today | Known: the menu uses the Today plan date; the row's check stamps yesterday correctly |
