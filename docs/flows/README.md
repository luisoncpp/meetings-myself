# Flow Docs

Operational guides organized by user or system action: "when this happens, everything that follows is this."

## Purpose

Use `docs/flows/` when you need to follow behavior end-to-end from a trigger instead of from a subsystem boundary.

This folder is for:

- debugging a concrete user action
- understanding which functions run in sequence
- finding which state is read, written, or only projected for UI
- locating side effects quickly without codebase-wide search

## How this differs from other doc types

- `docs/architecture/`
  - explains what a subsystem is, its design, and its invariants
- `docs/flows/`
  - explains what happens when an action occurs
- `docs/lessons-learned/`
  - explains counter-intuitive facts discovered while working in the area
- `docs/plan/`
  - explains how to change or refactor something

## Recommended format

Each flow doc should try to include:

1. Trigger
2. Entry point
3. Step-by-step sequence
4. Reads
5. Writes
6. Side effects
7. Files to inspect
8. Common failure modes

Keep these docs operational. Prefer short tables, explicit file names, and sequence lists over long essays.

| File | Scope |
|------|-------|
| [opening-the-app.md](opening-the-app.md) | Launch → device settings → DB open → health assess → writer lock |
| [opening-todays-plan.md](opening-todays-plan.md) | App open / date rollover → materialize → load-or-create plan → project |
| [archiving-an-entity.md](archiving-an-entity.md) | Library archive → lifecycle mutate → no cascade, no delete |
| [archiving-a-habit-already-in-a-plan.md](archiving-a-habit-already-in-a-plan.md) | Archive Habit in Library → plan entry stays, `archived: true`, still checkable |
