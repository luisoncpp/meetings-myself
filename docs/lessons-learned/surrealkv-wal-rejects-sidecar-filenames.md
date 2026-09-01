# SurrealKV WAL filenames are a closed set

**Date:** 2026-08-31

## What to know before starting

SurrealKV's WAL opener (`list_segment_ids`) walks `planning-db/wal/` and parses **every** file name as `{id:020}.wal`. Any other name is `Invalid segment name format`. The engine does not skip unknown files.

Windows Explorer writes `desktop.ini` into folders the user opens — especially cloud-synced folders on Windows 11. Google Drive / OneDrive can also leave ` (1)` copies **inside** `wal/`, not only next to `writer.lock`. Either one prevents `Surreal::new` from succeeding.

A release build has no console. If `PlanningApp::start` returns `Err` and `lib.rs` `.expect`s it, the process exits with no window and no Task Manager footprint. Treat a refused directory as `StoreHealth::Unreadable` and keep the UI up.

Strip only known OS sidecars before open. Do not delete unrecognized WAL names — those may be Drive conflict copies the conflict scanner should surface, or real corruption the user must restore.

## What this buys

- Launch still works when Explorer pollutes `wal/`.
- Sharing violations while Drive uploads a just-created database can be retried instead of failing the first folder pick.
- Nested conflict scan sees `planning-db/wal/*.wal (1)`.
