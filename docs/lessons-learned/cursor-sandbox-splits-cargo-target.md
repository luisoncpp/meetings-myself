# Cursor sandbox splits the Cargo target directory

**Date:** 2026-08-08

## What to know before starting

Cursor agent shells can inject `CARGO_TARGET_DIR` pointing at
`%TEMP%\cursor-sandbox-cache\<hash>\cargo-target`. That path is separate from
the workspace `./target`. SurrealDB with a native KV engine (especially
RocksDB) then cold-compiles into Temp on every new sandbox hash — measured at
~15–20 minutes and multi‑GB artifacts — while `./target` also keeps stale
fingerprint copies from earlier builds.

This project once held **~39 GB** in `./target/debug` plus **~15 GB** in a
sandbox cargo-target at the same time (~54 GB total).

## Rules for agents

1. Run every `cargo` / `npm run check:rust` / Tauri Rust build with unrestricted
   permissions so the sandbox does **not** redirect `CARGO_TARGET_DIR`.
2. Prefer the workspace `./target` (default when the env var is unset).
3. Do not `cargo clean` casually; reclaim disk with a deliberate clean only when
   fingerprint junk has piled up (multiple `surrealdb-librocksdb-sys-*` build
   dirs, multi‑GB stale `.lib` copies).

## Quick check

```powershell
echo $env:CARGO_TARGET_DIR
# Expected when compiling for this repo: empty / unset
```

If it points under `cursor-sandbox-cache`, stop and re-run unsandboxed.

## Related config

`.cargo/config.toml` reduces debug-info bloat for dependency crates. It cannot
override an injected `CARGO_TARGET_DIR` — the env var always wins.
