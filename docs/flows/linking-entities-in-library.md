# Linking Entities in Library

User-driven association creation and visualization directly on entity cards in the Library surface.

## Trigger

User views an active entity (Value, Goal, Habit, Task) in the Library and either:
1. Clicks the "Link to…" (`Vincular a…`) button on an entity card to create a new association.
2. Clicks the `×` button on an association tag to remove an existing link.

## Entry Point

- Tag list & Link button: `src/lib/surfaces/library/Private/AssociationTags.svelte`
- Modal dialog: `src/lib/planning-actions/Private/LinkModal.svelte`
- Row integration: `ValueRow.svelte`, `GoalRow.svelte`, `HabitRow.svelte`, `TaskRow.svelte`

## Step-by-Step Sequence

### Creating a Link
1. User clicks "Link to…" on an entity row (e.g., a Task).
2. Row opens `LinkModal` with `fromEnd = { kind, id }`, `fromTitle`, and `view`.
3. `LinkModal` resolves allowed target kinds sorted in priority order: `Goal` → `Habit` → `Value` → `Task`.
4. Tabs filter to only permitted domain pairs:
   - **Value**: `Goal`
   - **Goal**: `Habit`, `Value`, `Task`
   - **Habit**: `Goal`, `Task`
   - **Task**: `Goal`, `Habit`
5. The first available tab is selected by default.
6. The candidate list shows active entities of the selected tab kind that are not already linked to `fromEnd`.
7. User clicks "Link" (`Vincular`) on a candidate.
8. Modal calls `onlink(toEnd)` which delegates to `LibraryStore.link(fromEnd, toEnd)`.
9. `LibraryStore.link` invokes Rust IPC `link`, then calls `#change()` → `this.load()`.
10. `LibraryView` is re-fetched with updated `associations`.
11. `LinkModal` closes, and the entity row reactively renders the new association tag.

### Removing a Link
1. User clicks the `×` button on an association tag.
2. `AssociationTags` invokes `onunlink(associationId)` → `LibraryStore.unlink(associationId)`.
3. `LibraryStore.unlink` invokes Rust IPC `unlink(associationId)`, which sets the association lifecycle to `archived`.
4. Store reloads `LibraryView`, removing the tag from active view immediately.

## Reads

- `LibraryView.associations`: All active links returned by Rust `PlanningApp::library`.
- Entity views (`view.values`, `view.goals`, `view.habits`, `view.tasks`) to resolve titles and candidate lists.

## Writes

- Creates or archives an `Association` record in SQLite (`association` table).

## Files to Inspect

- `crates/planning-app/src/private/associations.rs`: Rust backend active associations querying and lifecycle management.
- `src/lib/planning-actions/Private/associations.ts`: Target order and unlinked candidate filtering helpers.
- `src/lib/planning-actions/Private/LinkModal.svelte`: Modal component with tabbed interface.
- `src/lib/surfaces/library/Private/AssociationTags.svelte`: Tag rendering with tooltip and unlink button.
