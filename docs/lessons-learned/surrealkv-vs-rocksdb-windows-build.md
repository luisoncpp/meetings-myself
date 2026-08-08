# SurrealKV vs RocksDB for embedded SurrealDB (Windows)

**Date:** 2026-08-08

## What to know before starting

For this app’s workload (embedded SurrealDB 3.2, local folder in a sync directory),
**SurrealKV is the better default on Windows** unless RocksDB is required for a
specific operational reason. Cold `cargo check` time is dominated by
`surrealdb-core` either way; RocksDB uniquely adds C++ compile cost, LLVM/bindgen,
multi‑hundred‑MB native libs, and fingerprint junk that ballooned `target/` to tens
of GB.

## Probe (this machine, 2026-08-08)

Minimal crates: `surrealdb 3.2.4` + `aws-lc-sys` `prebuilt-nasm`, `.cargo` profile
with `debug = 1` / dep `debug = 0`. Parallel cold checks (CPU contended):

| Engine | Feature | Cold `cargo check` | `target/` size | Incremental re-check |
|--------|---------|--------------------|----------------|----------------------|
| SurrealKV | `kv-surrealkv` | ~8m 00s | **0.62 GB** | ~1s |
| RocksDB | `kv-rocksdb` | ~8m 05s | **1.58 GB** | ~0.6s |

RocksDB-only native weight in the probe: ~836 MB under
`surrealdb-librocksdb-sys` (including ~288 MB `rocksdb.lib` + ~288 MB
`librocksdb.a`). SurrealKV had **no** rocksdb/bindgen build dirs.

API is drop-in:

```rust
// RocksDB
Surreal::new::<RocksDb>(path)
// SurrealKv
Surreal::new::<SurrealKv>(path)
```

Both are file-based single-node engines (`surrealkv://…` / `rocksdb://…`). On-disk
formats are **not** interchangeable — switching engines requires a fresh
`planning-db/` (acceptable before any real sync data exists).

## Why RocksDB hurt this repo

1. **Cursor sandbox** redirected `CARGO_TARGET_DIR` → cold native rebuilds into Temp
   while `./target` kept stale copies (see
   `cursor-sandbox-splits-cargo-target.md`).
2. **Fingerprint churn** left **8** `surrealdb-librocksdb-sys-*` build dirs; each
   full-debug copy was ~1.2 GB `.lib` + `.a` before the lighter `dev` profile.
3. **Tooling**: RocksDB needs LLVM/`LIBCLANG_PATH`; both engines still pull
   `aws-lc-sys` via `jsonwebtoken` (keep `prebuilt-nasm`).

## Recommendation

Amend ADR 0001 to **SurrealKV**, change `planning-store` to `kv-surrealkv` /
`SurrealKv`, drop RocksDB-specific LOCK sleep notes only after re-testing reopen
behavior, and keep the sandbox/`CARGO_TARGET_DIR` rules regardless of engine.

**Status (2026-08-08):** Adopted — `planning-store` uses `kv-surrealkv` /
`SurrealKv`. ADR 0001 amended.

Do **not** expect SurrealKV to make a cold `surrealdb-core` compile instant —
only to remove the RocksDB-shaped disk and native-toolchain tax.
