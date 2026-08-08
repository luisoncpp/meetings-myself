# Current Status

## Implemented

- Product language and the storage decision are documented.
- The implementation roadmap is in [`docs/plans/0001-self-planning-app.md`](../plans/0001-self-planning-app.md).
- CRUD semantics for the planning domain are decided and recorded in `CONTEXT.md` and the plan: a Library surface as the canonical management home, archive-only reversible lifecycle with no hard delete, forward-only propagation to existing plans, factory-model Recurring Task occurrences, preserved Associations, and fully correctable outcomes.
- **App shell** — Tauri 2 + Svelte 5 scaffold with `app_version` IPC, design tokens, quality gates (`npm run check`, `fallow audit`), and architecture tests. See [`docs/architecture/app-shell.md`](../architecture/app-shell.md).
- **Storage & sync safety** — `planning-core`, `planning-store`, and `planning-app` crates; embedded SurrealDB/SurrealKV in the Synchronization Folder; device-local settings; home time zone; `StoreHealth` gating and writer lock. See [`docs/architecture/storage.md`](../architecture/storage.md) and [`docs/flows/opening-the-app.md`](../flows/opening-the-app.md).
- **Planning domain & Library API** — Values, Goals, Habits, Tasks, Associations; orthogonal lifecycle; generic `Records` gateway; Library creation and lifecycle use cases; read-model views with projected overdue/archived; Tauri commands and TypeScript domain mirror. See [`docs/architecture/planning-domain.md`](../architecture/planning-domain.md) and [`docs/flows/archiving-an-entity.md`](../flows/archiving-an-entity.md). Plan: [`docs/plans/done/0004-planning-domain.md`](../plans/done/0004-planning-domain.md).
- **Daily planning** — Daily Plans with ordered task selection; Weekly Focus; pinned-habit seeding at plan creation; Habit Check-ins; idempotent Recurring Task materialization; record keys as invariants; plan read models with projected archived/unpinned flags. See [`docs/architecture/daily-planning.md`](../architecture/daily-planning.md), [`docs/flows/opening-todays-plan.md`](../flows/opening-todays-plan.md), and [`docs/flows/archiving-a-habit-already-in-a-plan.md`](../flows/archiving-a-habit-already-in-a-plan.md). Plan: [`docs/plans/done/0005-daily-plan-and-habits.md`](../plans/done/0005-daily-plan-and-habits.md).
- **Weekly reviews & reports** — `planning-reports` crate for domain-blind Markdown files; app-owned summary region with preservation contract; `WeeklySummary` computed fresh on every open; Weekly Review use cases with prior-report read and next-week focus ensure; Tauri commands and TypeScript mirror. See [`docs/architecture/weekly-reports.md`](../architecture/weekly-reports.md), [`docs/flows/opening-a-weekly-review.md`](../flows/opening-a-weekly-review.md), and [`docs/lessons-learned/app-owned-regions-in-user-owned-files.md`](../lessons-learned/app-owned-regions-in-user-owned-files.md). Plan: [`docs/plans/done/0006-weekly-review-and-reports.md`](../plans/done/0006-weekly-review-and-reports.md).
- **UI surfaces** — Daily Plan (main window home), Library (main window tab), Weekly Review (separate window); store-class pattern; `src/lib/api` IPC boundary; pessimistic mutations with optimistic reorder only; health gate in shell. See [`docs/architecture/ui-surfaces.md`](../architecture/ui-surfaces.md), [`docs/flows/completing-a-task-from-the-daily-plan.md`](../flows/completing-a-task-from-the-daily-plan.md), and [`docs/lessons-learned/svelte-5-runes-need-the-svelte-ts-extension.md`](../lessons-learned/svelte-5-runes-need-the-svelte-ts-extension.md). Plan: [`docs/plans/done/0007-ui-surfaces.md`](../plans/done/0007-ui-surfaces.md).

## Known gaps (Plan 0007)

- Manual Windows accessibility pass still pending (automated guards and component tests in place; see `docs/architecture/ui-surfaces.md`).
- In-place title editing not wired (no IPC yet).
- Goal target-date editing in Library is create-only (no update IPC).
- Recurring Tasks section missing from Library (deferred).

## Next

- Plan 0008: Launcher — [`docs/plans/0008-launcher.md`](../plans/0008-launcher.md).
