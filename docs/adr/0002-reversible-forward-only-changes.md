# ADR 0002: Favor reversible, forward-only changes over destructive ones

## Status

Accepted

## Context

Values, Goals, Habits, Tasks, and their Associations are editable at any time from the Library surface, not only during the Weekly Review. The app is local-first with Google Drive synchronization, so an action on one device must never destructively conflict with state produced on another. The accumulated personal history — Habit Check-ins, Task outcomes, Weekly Reports — is the app's long-term value.

## Decision

No entity is ever hard-deleted. Values, Goals, Habits, and Tasks are archived reversibly and can be restored from the Library. An archived entity remains, marked, in any Daily Plan or Weekly Focus that already contains it and can still be completed there, but it cannot be newly selected.

Lifecycle and schedule changes apply forward-only: existing Daily Plans and Weekly Focuses are never rewritten. Unpinning, Habit Cadence changes, and archiving take effect from the next plan or selection.

A Recurring Task rule is a factory: once materialized, an occurrence is an ordinary Task unaffected by later rule edits, and archiving the rule only stops future materialization.

Associations never cascade: archiving one side leaves the other untouched and preserves the link, dormant until the archived side is restored.

Recorded outcomes stay correctable: completed Tasks can be reopened and past Habit Check-ins changed at any time. Weekly Report summaries always reflect the latest data; only the typed reflection is preserved as written.

## Consequences

- The data model needs archived states and marked plan entries; there are no cascading delete paths anywhere.
- Weekly Report summaries are regenerated from current data, never frozen.
- Entities accumulate in the archive instead of being deleted, so the Library must keep archived items out of everyday views while remaining restorable.
- A future request for hard delete, plan-rewriting archive, or locked outcomes is a deliberate deviation from this ADR, not an overlooked improvement.

## Considered alternatives

- **Hard delete for history-free entities:** rejected; the "has history" test becomes a subtle invariant everywhere a delete action renders, and mistakes under sync are unrecoverable.
- **Immediate withdrawal on archive (remove from today's plan and focus):** rejected; the app must not rewrite a plan the user already made, and mid-week lifecycle changes are valuable reflection signal in the Weekly Review.
- **A correction window that locks outcomes once the week's report exists:** rejected; it invents a locked-state invariant and makes reports diverge silently from the underlying data.
