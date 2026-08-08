# Architecture Docs

Canonical technical guides — the single source of truth for each subsystem's design, data model, and behavior rules.

Covers only what's already implemented. For architecture docs of not implemented yet, check `docs/plans`

| File | Subsystem | Notes |
|------|-----------|-------|
| [app-shell.md](app-shell.md) | Tauri + Svelte shell, crate graph, quality gates | Enforced by `tests/architecture.test.ts` |
| [storage.md](storage.md) | `planning-core` / `planning-store` / `planning-app`, sync folder layout, `StoreHealth`, writer lock | Plan 0003 |
| [planning-domain.md](planning-domain.md) | Values, Goals, Habits, Tasks, Associations, Library views, `Records` gateway | Plan 0004 |
