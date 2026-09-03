# Switching the Synchronization Folder

## Trigger

User clicks **Switch folder** in the top navigation bar during normal use, or clicks **Choose a different folder** on the health gate banner when the configured folder is missing.

## Entry point

- Main UI: `src/lib/shell/Private/Navigation.svelte` (`onswitchFolder`) → `AppShell.svelte` (`chooseReplacementFolder`)
- Health gate: `src/lib/shell/Private/HealthBanner.svelte` (`onchooseFolder`) → `AppShell.svelte` (`chooseReplacementFolder`)

## Steps

1. **Pick directory** — `pick_sync_folder` opens native folder picker. If user cancels, stop.
2. **Update settings & reconnect** — `choose_sync_folder` receives new path:
   - Sets `sync_folder` in `device-settings.json` and persists it.
   - Drops any existing `WriterLock` (removing `writer.lock` in the old folder).
   - Drops the existing database connection.
   - Reconnects to the new folder: assesses conflicts, connects to SurrealDB, loads `settings:home`.
   - Acquires `writer.lock` if `Ready`.
3. **Update UI state**:
   - `AppShell` receives updated `StoreHealth`.
   - If `setupIncomplete` (e.g. `NoHomeZone` on a fresh folder), `AppShell` renders `Setup` for zone selection.
   - If `ready`, `folderRevision` increments and child views (`DailyPlan` / `Library`) remount with clean state for the new folder.
   - If blocked (`folderMissing`, `syncConflict`, `lockedByAnotherDevice`, `unreadable`), `HealthBanner` displays the status.

## Reads

| Source | What |
|--------|------|
| Native dialog | Selected folder path |
| New folder | Conflict scan, SurrealDB `settings:home`, `writer.lock` |

## Writes

| Target | When |
|--------|------|
| `device-settings.json` | Immediately on folder choice |
| Old `<sync-folder>/writer.lock` | Removed when lock is dropped |
| New `<sync-folder>/writer.lock` | Created if health is `Ready` |

## Side effects

- Disconnects previous folder and frees its writer lock.
- Connects to new folder and remounts active UI view.

## Files to inspect

| Path | Role |
|------|------|
| `src/lib/shell/Private/Navigation.svelte` | Switch folder button |
| `src/lib/shell/Private/AppShell.svelte` | Triggers picker, reconnects, increments revision |
| `crates/planning-app/src/private/setup.rs` | `choose_sync_folder`, `reconnect` |
| `src-tauri/src/private/commands.rs` | Tauri commands |
