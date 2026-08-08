# Weekly Reports

One canonical Markdown file per Calendar Week in the Synchronization Folder. The app owns YAML front matter and one HTML-comment-delimited summary region; everything else is user prose. Extends daily planning from [daily-planning.md](daily-planning.md) (Plan 0006).

## Crate split

```
planning-app ──► planning-reports   (domain-blind: parse, markers, paths)
     │
     └── weekly_summary.rs, summary_markdown.rs, weekly_review_use_cases.rs
```

`planning-reports` knows nothing about Tasks, Habits, or the database. `planning-app` computes a `WeeklySummary` from current data, renders it to Markdown, and hands the text to `WeeklyReportFile::write`.

## File path — one report per week

```
<sync>/weekly-reports/{YYYY-Www}-weekly-report.md
```

Example: `weekly-reports/2026-W32-weekly-report.md`

The filename is derived from the week label. Reopening a review always reads and writes the same path — there is no second file to create. Same mechanism as record keys in [daily-planning.md](daily-planning.md).

## Annotated example

```markdown
---                              ← app-owned YAML front matter (replaced wholesale on write)
week: 2026-W32
week_start: 2026-08-03           ← snake_case for human editors (IPC uses camelCase)
week_end: 2026-08-09
schema: 1                        ← forward-compat gate (see below)
generated_at: 2026-08-09T18:22:11Z
---

<!-- self-planning:summary:start -->   ← app-owned region start
## Week in review

**Completed:**
- Prepare portfolio
...
<!-- self-planning:summary:end -->     ← app-owned region end

## Reflection                         ← user-owned — preserved byte for byte

It was a good week.
```

## Front matter fields

| Field | Meaning |
|-------|---------|
| `week` | ISO week label, `"2026-W32"` |
| `week_start` | Monday of the week |
| `week_end` | Sunday of the week |
| `schema` | Format version; `ReportFrontMatter::SCHEMA == 1` |
| `generated_at` | UTC timestamp of last summary regeneration |

Serialized `snake_case` so a human editing the file by hand sees `week_start`, not `weekStart`.

## Preservation contract

On every write, the front matter is replaced wholesale and the region between
`<!-- self-planning:summary:start -->` and `<!-- self-planning:summary:end -->` is replaced
wholesale. **Every other byte of the file is preserved exactly**, including whitespace, the
user's own headings, and anything they added below, above, or between.

If markers are missing or corrupt, `SummaryBlock::replace` prepends a new block without rewriting or dropping existing prose.

## Summaries are never stored

`PlanningApp::weekly_summary` queries Tasks, Habits, Check-ins, and Goals for the week and returns a fresh `WeeklySummary` on every call. Nothing about the summary is persisted except the rendered Markdown inside the comment region.

Reopening a three-week-old review shows corrected check-ins and reopened Tasks — no migration.

Rendered text is counts and titles only (no percentages, streaks, or scores).

## Schema forward-compatibility

`schema` in front matter is a version gate. If `schema` is **greater than** `ReportFrontMatter::SCHEMA`, parse returns `UnsupportedSchema` — the file is refused, never silently overwritten. Older schemas the app understands are accepted.

## Application API — `planning-app`

| Area | Module | Examples |
|------|--------|----------|
| Open review | `weekly_review_use_cases.rs` | `open_weekly_review`, `open_current_review` |
| Summary | `weekly_summary.rs` | `weekly_summary` |
| Render | `summary_markdown.rs` | `render` |
| Reflection | `weekly_review_use_cases.rs` | `save_reflection` |
| Path | `weekly_review_use_cases.rs` | `report_path` |

All writes go through `require_reports()` — refused unless `StoreHealth` is `Ready` and a sync folder is connected.
