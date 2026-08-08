# Orthogonal Lifecycle Beats a Single State Enum

**Date:** 2026-08-08

## What looks intuitive

`CONTEXT.md` describes a Task as "open, completed, or archived" — three words that invite a single three-variant enum.

## What breaks

One enum forces an arbitrary choice when a Task is **archived while completed**. On restore, the system must guess: should it come back open or completed? Callers would re-derive history from logs or heuristics.

Goals have the same tension between "pursuing / achieved" and "archived".

## What we did

Two independent fields on each entity:

- **Outcome axis:** `Completion` (Task) or `Achievement` (Goal)
- **Presence axis:** `Lifecycle` (`Active` | `Archived`)

The UI still shows a single `TaskState` — but only as a **projection** in read models, with archived taking precedence.

## Generalizable lesson

When users describe combinable states ("archived *and* completed"), they are usually naming **orthogonal axes**, not mutually exclusive variants. Model the axes; derive display labels at the edge.
