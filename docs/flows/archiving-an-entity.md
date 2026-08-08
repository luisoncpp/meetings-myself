# Archiving an Entity

## Trigger

User archives a Task (or Value, Goal, or Habit) from the Library.

## Entry point

`src/lib/api/index.ts` → `archiveEntity(end)` → Tauri `archive_entity` → `planning-app` archive method for that entity kind.

## Steps

1. **IPC** — frontend passes an `AssociationEnd` (`{ kind: 'task', id: '…' }`).
2. **Dispatch** — `lifecycle_commands.rs` matches the end and calls `archive_task`, `archive_goal`, `archive_habit`, or `archive_value`.
3. **Load** — `PlanningApp::mutate` loads the record via `Records::find`; missing id → `AppError::NotFound`.
4. **Mutate** — set `lifecycle` to `Archived`. Task `completion` / Goal `achievement` are **not** changed.
5. **Save** — `Records::save` upserts the document back.
6. **Read model** — next `library()` call hides the entity unless `includeArchived: true`; view sets `archived: true` and `TaskState::Archived` where applicable.

Restore is the same path through `restore_entity`, setting `lifecycle` back to `Active`.

## Reads

| Source | What |
|--------|------|
| SurrealDB `task` / `goal` / `habit` / `value` table | Current entity document |
| Home settings (indirect) | Only for completion/achievement dates on complete/achieve — not for archive itself |

## Writes

| Target | What changes |
|--------|--------------|
| Entity record | `lifecycle: Archived` only |

## Side effects

**None.**

Specifically, archiving does **not**:

- Delete the record or any Association
- Archive linked entities on the other side of an Association
- Change Task completion, Goal achievement, Habit cadence, or pinned state
- Rewrite an existing Daily Plan or Weekly Focus (plan 0005 marks archived entries at read time)

Associations remain in the database with `lifecycle: Active`. `associations_for` still returns them.

## Files to inspect

| Path | Role |
|------|------|
| `src-tauri/src/private/lifecycle_commands.rs` | `archive_entity`, `restore_entity` |
| `crates/planning-app/src/private/entity_lifecycle.rs` | Task/Goal archive + completion |
| `crates/planning-app/src/private/habit_lifecycle.rs` | Habit/Value archive |
| `crates/planning-app/src/private/views.rs` | Library projection of `archived` / `TaskState` |
| `crates/planning-store/src/private/records.rs` | Upsert persistence |

## Common failure modes

| Symptom | Cause |
|---------|-------|
| Command error "not ready" | Setup incomplete or sync conflict — `require_database` refused |
| "no task with id …" | Stale UI id or record never saved |
| Link disappears from UI but still in DB | `unlink` archives the **Association**, not the entity — check `all_associations` vs `associations_for` |
| Restored Task came back completed | Expected — completion is orthogonal to lifecycle |
