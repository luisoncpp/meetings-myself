# Opening the App

## Trigger

User launches the desktop application (or the app restarts after choosing a sync folder).

## Entry point

`src-tauri/src/lib.rs` → `PlanningApp::start(StartRequest { settings_path, clock })`

## Steps

1. **Load device settings** — `DeviceSettingsFile::load()` reads or creates `device-settings.json` in the OS config directory.
2. **Assess without a folder** — if `sync_folder` is `None`, health is `SetupIncomplete { NoSyncFolder }`; stop here.
3. **Open database** — if the folder path exists, `Database::open` creates `planning-db/` and connects SurrealDB/SurrealKV.
4. **Load home zone** — `HomeSettingsRepository::load` reads `settings:home`; zone may still be `None`.
5. **Assess health** — `StoreHealth::assess` checks folder presence, conflict artifacts, and whether home zone is set.
6. **Acquire writer lock** — if health is `Ready`, `WriterLock::acquire` writes or refreshes `writer.lock`.

`reconnect()` reruns steps 3–6 after folder selection or sync recovery (plan 0008).

## Reads

| Source | What |
|--------|------|
| `device-settings.json` | `sync_folder`, device identity, launcher prefs |
| `<sync-folder>/planning-db/` | SurrealDB — `settings:home` record |
| `<sync-folder>/` + `planning-db/` | Conflict artifact scan (one level) |
| `<sync-folder>/writer.lock` | Lock holder and heartbeat timestamp |

## Writes

| Target | When |
|--------|------|
| `device-settings.json` | First run (defaults), `choose_sync_folder` |
| `planning-db/` | First open (directory + SurrealKV files) |
| `settings:home` | `set_home_zone` |
| `writer.lock` | Health is `Ready` after assess or zone set |

## Side effects

- Creates `planning-db/` under the Synchronization Folder on first open.
- Creates `writer.lock` when setup is complete and no fresher lock blocks.
- Removes `writer.lock` on clean app shutdown (`WriterLock::drop`).

## Files to inspect

| Path | Role |
|------|------|
| `src-tauri/src/lib.rs` | Tauri entry; calls `PlanningApp::start` |
| `crates/planning-app/src/private/setup.rs` | `start`, `reconnect`, `choose_sync_folder`, `set_home_zone` |
| `crates/planning-app/src/private/service.rs` | `health`, `calendar`, `require_database`, `take_lock` |
| `crates/planning-store/src/private/device_settings.rs` | Device settings file |
| `crates/planning-store/src/private/database.rs` | SurrealDB open |
| `crates/planning-store/src/private/health.rs` | `StoreHealth::assess` |
| `crates/planning-store/src/private/writer_lock.rs` | Writer lock acquire/release |
| `src/lib/api/index.ts` | Frontend `storeHealth`, `chooseSyncFolder`, `setHomeZone` |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| `setupIncomplete / NoSyncFolder` | Fresh install or `sync_folder` cleared from device settings |
| `setupIncomplete / NoHomeZone` | Folder chosen but time zone not yet set |
| `folderMissing` | Drive folder not mounted yet (laptop woke before sync client) |
| `syncConflict` | Drive duplicated a file — `CURRENT (1)` or `(conflicted copy …)` in sync folder or `planning-db/` |
| `lockedByAnotherDevice` | Another machine has a fresh `writer.lock` (< 15 min old) |
| `calendar()` errors despite `ready` | Should not happen — indicates a logic bug |
| Database open fails immediately after close | Embedded engine lock not yet released — retry after ~100 ms |
