# Current Status

## Implemented

- Product language and the storage decision are documented.
- The implementation roadmap is in [`docs/plans/0001-self-planning-app.md`](../plans/0001-self-planning-app.md).
- CRUD semantics for the planning domain are decided and recorded in `CONTEXT.md` and the plan: a Library surface as the canonical management home, archive-only reversible lifecycle with no hard delete, forward-only propagation to existing plans, factory-model Recurring Task occurrences, preserved Associations, and fully correctable outcomes.
- **App shell** — Tauri 2 + Svelte 5 scaffold with `app_version` IPC, design tokens, quality gates (`npm run check`, `fallow audit`), and architecture tests. See [`docs/architecture/app-shell.md`](../architecture/app-shell.md).

## Next

- Plan 0003: workspace crates (`planning-core`, `planning-store`, etc.) and embedded SurrealDB.
