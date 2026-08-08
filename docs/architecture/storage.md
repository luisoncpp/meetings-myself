# Storage & Sync Safety

Embedded SurrealDB on SurrealKV inside a Google Drive Synchronization Folder, with device-local settings kept outside sync and a health gate that refuses writes until the folder is trustworthy.

## Crate graph

One-way dependency — binaries never reach into crate internals:

```
src-tauri / launcher ──► planning-app ──► planning-store ──► planning-core
                              │                  │
                              └──────────────────┘
```

| Crate | Responsibility |
|-------|----------------|
| `planning-core` | Pure time and identity vocabulary — `Clock`, `HomeCalendar`, `CalendarWeek`, id newtypes. No IO. |
| `planning-store` | SurrealDB connection, device settings file, home settings record, conflict scan, writer lock, `StoreHealth`. |
| `planning-app` | Narrow application API — `PlanningApp::start`, setup commands, `calendar()`. The **only** crate `src-tauri` and `launcher` may depend on. |

## Synchronization Folder layout

```
<sync-folder>/
  planning-db/          # SurrealDB/SurrealKV directory (synchronized)
  writer.lock           # Cooperative one-active-writer marker (JSON heartbeat)
  weekly-reports/       # Markdown weekly reports (plan 0006 — not yet created)
```

`planning-db/` is created on first open. SurrealKV stores a directory of files, not a single `.db` file — Google Drive syncs the whole tree.

## Device settings (outside sync)

Per-device facts live in the OS config directory (`device-settings.json` via `directories`), never inside the Synchronization Folder:

- `device_id`, `device_name`
- `sync_folder` path (pointer to the Drive folder)
- launcher timing (`launch_time`, `retry_window_minutes`, `last_missed_prompt`)

Each machine has its own file. Only planning data and the home time zone travel through Drive.

## Home time zone

Stored in SurrealDB as `settings:home` (`HomeSettingsRepository`). Starts **unset** — setup is explicitly incomplete until the user picks a zone. No silent UTC default.

`HomeCalendar` is the only source of dates. Every date calculation goes through `calendar().today(clock)` / `current_week(clock)`; nothing calls `chrono::Local` or the device time zone.

## StoreHealth gate

`StoreHealth` is a returned value, not an exception. Only `Ready` permits writes.

| Variant | Meaning |
|---------|---------|
| `Ready` | Folder present, no conflicts, home zone set — writes allowed |
| `SetupIncomplete` | `NoSyncFolder` or `NoHomeZone` |
| `FolderMissing` | Configured path not mounted (Drive still syncing?) |
| `LockedByAnotherDevice` | Fresh `writer.lock` held by another device |
| `SyncConflict` | Drive conflict artifacts detected |
| `Unreadable` | IO or parse failure |

Assessment order is most-blocking first: missing folder beats conflict beats unset zone. `require_database()` and `calendar()` return `Err` unless health is `Ready` — no write path can skip the check.

## Writer lock

Google Drive provides no real file locking. `writer.lock` is a cooperative advisory marker:

- JSON record: `device_id`, `device_name`, `heartbeat_at`
- **15-minute staleness window** (`STALE_AFTER_MINUTES = 15`): a crashed device leaves a stale lock another device can take over
- Same device reacquires its own lock on restart
- `Drop` removes the file on clean shutdown (best effort)

Deleting `writer.lock` is removal of a transient marker, not planning data — ADR 0002's no-hard-delete rule covers entities, not lock files.

## Conflict detection

`conflicts::scan` walks the sync folder and `planning-db/` one level deep. Two Drive rename patterns flag `SyncConflict`:

- `(conflicted copy …)` suffix (Google Drive desktop)
- ` (N)` before extension — e.g. `CURRENT (1)`, `MANIFEST-000004 (2)`

Any match blocks writes until the user resolves duplicates manually. Patterns are engine-agnostic (Drive renames whatever files live under `planning-db/`).

## SurrealDB API notes

Worth remembering when extending `planning-store`:

| Quirk | What we do |
|-------|------------|
| Path argument | `Surreal::new::<SurrealKv>(path.to_string_lossy().as_ref())` — takes `&str`, not `Path` |
| Record types | Derive `surrealdb::types::SurrealValue` alongside `Serialize`/`Deserialize` |
| `Tz` storage | Store as IANA string (`zone.name()`), parse with `Tz::from_str` on load |
| Missing table | `select` on a table that does not exist yet returns an error containing `"does not exist"` — treat as unset, return defaults |
| Lock after drop | After `drop(database)`, the embedded engine may release its on-disk lock asynchronously; tests sleep 100 ms (or retry) before reopening |
