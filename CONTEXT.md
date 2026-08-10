# Self-Planning Context

The language of the local-first personal planning app. It distinguishes daily action, weekly reflection, and the enduring motivations that connect them.

## Language

**Daily Plan**:
A fresh, editable ordered selection of Tasks and Habits for one calendar day. It is not a time-blocked schedule, and later changes to its Tasks and Habits never rewrite it — they take effect from the next plan.
_Avoid_: TODO-today, daily checklist

**Weekly Review**:
A guided self-meeting for a Calendar Week that reflects on the prior week and prepares the coming one. It grants no exclusive powers — the same changes can be made in the Library at any time.
_Avoid_: weekly planning, weekly meeting

**Calendar Week**:
The Monday-through-Sunday unit to which a Weekly Review and Weekly Report belong.

**Weekly Report**:
The one canonical written record of a Calendar Week, combining preserved reflection with always-current summaries of Task outcomes and Habit Check-ins.
_Avoid_: weekly notes, weekly summary

**Weekly Focus**:
The flexible shortlist of open Tasks selected for a coming Calendar Week. It guides Daily Plan creation without scheduling Tasks to particular days, and it can be adjusted at any time, not only during the Weekly Review.

**Library**:
The standing collection of all Values, Goals, Habits, and Tasks, curated at any moment independently of any Daily Plan or Weekly Review.
_Avoid_: management screen, backlog, registry

**Archive**:
The reversible resting state of a Value, Goal, Habit, or Task that is no longer in use: it remains, marked, in plans and focuses that already contain it, but cannot be newly selected, keeps its full history, and can be restored from the Library. Nothing is ever hard-deleted.
_Avoid_: delete, trash, remove

**Task Pool**:
The persistent collection of Tasks available for selection into Weekly Focus or a Daily Plan. A Task is poolable while it is active and either open or marked non–one-off (even when completed). One-off Tasks leave the pool when completed. Selecting into focus or a plan never removes a Task from the pool.

**One-off Task**:
A Task that leaves the Task Pool once completed. Non–one-off Tasks stay poolable while active so recurring work (e.g. monthly payments) remains easy to re-add. Defaults to one-off at creation.

**Task**:
An actionable item that is open, completed, or archived; completion is reversible. An unfinished Task remains available until deliberately archived.

**Recurring Task**:
A rule that produces independent Task occurrences on a recurrence schedule. Once materialized, an occurrence is an ordinary Task with its own outcome; editing or archiving the rule affects future occurrences only.

**Deadline**:
A date, without a time of day, by which a Task should be completed. A missed Deadline makes the Task Overdue without changing it automatically.

**Importance**:
The manual assessment of how much a Task matters: unclassified, low, or high.

**Urgency**:
The manual and deadline-informed assessment of a Task's time pressure: unclassified, low, or high.

**Value**:
An enduring personal principle served by one or more Goals. Values are active or archived; they are never completed.

**Goal**:
An intended outcome that can be active, achieved, or archived, and may have a target date. A Goal can serve multiple Values and be supported by multiple Habits and Tasks.

**Habit**:
A repeated practice with a manual Habit Strength. Habits are active or archived. A Habit can support multiple Goals and be advanced by multiple Tasks.

**Habit Strength**:
The qualitative stage of a Habit: Reminder-dependent, Cue-triggered, Strengthening, or Established.

**Habit Cadence**:
The days on which a Habit is due: every day or selected weekdays. Changes to a Habit Cadence apply from the next Daily Plan.

**Pinned Habit**:
A Habit automatically included in Daily Plans on the days defined by its Habit Cadence. Unpinning stops that automatic inclusion in future Daily Plans without deleting its history or removing it from plans that already exist.

**Habit Check-in**:
The record of a Habit on one day: Done, Skipped, or Not completed. It is distinct from Task completion and can be corrected for any past day.

**Association**:
A many-to-many relevance link between Values and Goals, Goals and Habits, or Tasks and Goals or Habits. An Association does not imply ownership: archiving one side never affects the other, and the link itself is preserved, dormant until the archived side is restored.

## Example dialogue

> **Planner:** “I added *prepare portfolio* to this week's Weekly Focus because it advances my *career transition* Goal.”
>
> **Domain expert:** “Good. Keep the Task in the Task Pool until it is completed or archived. Add it to today's Daily Plan only if you will work on it today. Your *writing practice* Habit is pinned, so it will already appear because today is one of its cadence days.”
>
> **Planner:** “On Wednesday I realized my *meditation* Habit needed a lighter Habit Cadence, so I changed it in the Library instead of waiting for the Weekly Review.”
