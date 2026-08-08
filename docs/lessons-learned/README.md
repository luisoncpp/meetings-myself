# Lessons Learned

Knowledge that helps future development: effective strategies, counter-intuitive facts, and patterns worth remembering across the codebase.

## When to Add

- When a strategy that seemed right turned out to be wrong or suboptimal.
- When something counter-intuitive was discovered through experimentation.
- When a workaround for external dependency behavior was needed and the reason isn't obvious from code.
- When a pattern proved effective and worth formalizing.

## How to Add

Create a new file in this directory named after the topic (e.g., `quill-bounds-always-relative-to-container.md`, `optimistic-ui-pattern-for-toggle-sync.md`). Then add it to the index below.

The entry should answer: **what is counter-intuitive or effective that I should know before starting similar work?**

Avoid: "bug description + fix". Prefer: "what I learned that applies to future work."

## Index

| File | Topic | Date |
|------|-------|------|
| [surrealdb-rocksdb-windows-build-prerequisites.md](surrealdb-rocksdb-windows-build-prerequisites.md) | Historical: RocksDB Windows LLVM/NASM blockers (engine is now SurrealKV) | 2026-08-08 |
| [cursor-sandbox-splits-cargo-target.md](cursor-sandbox-splits-cargo-target.md) | Cursor sandbox `CARGO_TARGET_DIR` forces full SurrealDB rebuilds | 2026-08-08 |
| [surrealkv-vs-rocksdb-windows-build.md](surrealkv-vs-rocksdb-windows-build.md) | SurrealKV vs RocksDB compile size/time on Windows; prefer SurrealKV | 2026-08-08 |
| [sync-safety-is-a-value-not-an-exception.md](sync-safety-is-a-value-not-an-exception.md) | `StoreHealth` as returned value, not per-write exceptions | 2026-08-08 |
| [orthogonal-lifecycle-beats-a-single-state-enum.md](orthogonal-lifecycle-beats-a-single-state-enum.md) | Model archive and outcome as orthogonal axes, not one enum | 2026-08-08 |
| [surrealdb-records-via-json-value.md](surrealdb-records-via-json-value.md) | Generic Records round-trips via `serde_json::Value` for SurrealDB 3 | 2026-08-08 |
| [record-keys-as-invariants.md](record-keys-as-invariants.md) | Record key is the uniqueness rule; `materialized_through` is a hint only | 2026-08-08 |
