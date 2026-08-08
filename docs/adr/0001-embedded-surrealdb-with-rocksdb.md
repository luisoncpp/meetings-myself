# ADR 0001: Use embedded SurrealDB with SurrealKV

## Status

Accepted

## Context

The app is local-first, needs document-shaped records with flexible links between values, goals, habits, and tasks, and synchronizes its data through a Google Drive for desktop folder. No separate database installation or server is acceptable. Editing occurs on one device at a time.

## Decision

Use SurrealDB embedded in the Tauri Rust process with its **SurrealKV** storage engine (`kv-surrealkv`). Store the database directory in the Synchronization Folder. Keep one canonical Weekly Report per Calendar Week as a separate Markdown file with YAML front matter in the `weekly-reports/` folder; name it deterministically (for example, `2026-W32-weekly-report.md`) and edit that file when revisiting the week. The Markdown body remains externally editable and is reread when opened; front matter remains stable and app-owned.

The app does not use SurrealDB's network-server, authentication, or cloud features. Google Drive remains responsible only for file synchronization. Before switching devices, the editing device must close the app and finish synchronizing.

Launcher settings such as the launch time, retry window, and synchronization-folder path are device-specific. Only planning data is synchronized.

Calendar Week and Daily Plan boundaries use a synchronized home time zone, not whichever time zone a device happens to report.

Recurring Task occurrences are materialized just in time through the current home-time-zone date. Generation is idempotent so reopening the app cannot create duplicates.

The Daily Plan Launcher defaults to a 7:00 AM first attempt in the home time zone and retries after computer startup or synchronization recovery during its configured morning window.

## Consequences

- The domain can use documents and explicit links without a separate service or user installation.
- The database is a synchronized directory, not a single file.
- The Daily Plan Launcher needs a stable, read-only way to determine whether a Daily Plan exists.
- Concurrent editing and unsynchronized device switching are unsupported.
- SurrealKV is pure Rust: no LLVM/bindgen or C++ RocksDB compile on Windows. Cold builds are still dominated by `surrealdb-core`.
- On-disk format is not compatible with RocksDB; existing `planning-db/` trees from the RocksDB era must be discarded or migrated (none expected in early development).

## Considered alternatives

- **SQLite:** mature and embedded, but relational modeling is not the preferred fit.
- **redb:** lightweight embedded key-value storage, but requires the app to implement document queries and relationship indexes.
- **BonsaiDb:** Rust-native document database, but remains alpha with an explicit data-loss warning.
- **NeDB:** Node.js-oriented and unsuitable for Tauri without a Node sidecar.
- **RocksDB (`kv-rocksdb`):** original choice; abandoned on Windows because native builds ballooned `target/` (tens of GB with fingerprint churn) and required LLVM. See `docs/lessons-learned/surrealkv-vs-rocksdb-windows-build.md`.

## Amendments

### 2026-08-08 — Switch storage engine to SurrealKV

Replaced RocksDB with SurrealKV for embedded persistence. API remains
`Surreal::new::<SurrealKv>(path)` under `planning-db/`. Filename of this ADR
(`…-with-rocksdb.md`) is historical; the decision above is authoritative.

### 2026-08-08 — Cooperative writer lock (plan 0003)

ADR 0001 required one active writer but did not specify detection. Implementation uses a `writer.lock` file in the Synchronization Folder: JSON with `device_id`, `device_name`, and `heartbeat_at`. Another device is refused while the heartbeat is fresher than 15 minutes; a stale lock (crash) can be taken over. Clean shutdown deletes the file. This is advisory — Google Drive provides no real locking — but sufficient for the "one device at a time" rule.

### 2026-08-08 — Home time zone starts unset (plan 0003)

The synchronized home time zone is stored in SurrealDB but begins as `None`. Setup is explicitly incomplete (`SetupIncomplete { NoHomeZone }`) until the user chooses a zone. No silent default to UTC or the device time zone.
