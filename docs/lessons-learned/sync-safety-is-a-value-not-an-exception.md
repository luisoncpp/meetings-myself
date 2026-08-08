# Sync safety is a value, not an exception

**Date:** 2026-08-08

## What to know before starting

`StoreHealth` is returned as data, not thrown as an error from each write path. The UI renders the exact fault; the launcher makes a go/no-go decision without catching exceptions; and no write path can forget the check because `require_database()` and `calendar()` return `Err(NotReady(health))` unless health is `Ready`.

## What this buys

- **UI precision** — `setupIncomplete`, `syncConflict`, and `lockedByAnotherDevice` each need different copy and actions. A generic `DatabaseError` would collapse them.
- **Launcher simplicity** — plan 0008 reads `store_health` and decides whether to open the Daily Plan window. Pattern matching on a value is clearer than exception handling across an IPC boundary.
- **Compile-time enforcement** — write paths take `&Database` only through `require_database()`, which checks `permits_writes()` first. You cannot accidentally call `upsert` while health is `SyncConflict`.

## The failure this prevents

The design targets a *silent* write against a half-synchronized SurrealDB directory. Drive may have merged two versions of a live file or left conflict copies alongside it. Opening SurrealDB on that tree might succeed — the KV engine does not know Drive renamed a sibling file. Without a pre-write health check, the app would happily persist new records on top of torn state. No exception type would have made that visible; the corruption would surface days later on another device.

Returning `StoreHealth::SyncConflict { artifacts }` before any write forces the user to resolve duplicates first.

## Counter-intuitive part

`Ready` is the only variant that permits writes, but assessment runs even when no database is open (setup incomplete). Health is about the *folder's trustworthiness*, not "did the last query succeed." A query error becomes `Unreadable`; a conflict artifact blocks readiness even if SurrealDB opens fine.
