# Opening a Weekly Review

## Trigger

The Weekly Review is requested — the UI loads the review surface (plan 0007), or the frontend calls `openCurrentReview()` / `openWeeklyReview(week)`.

## Entry point

`src/lib/api/index.ts` → `openCurrentReview()` / `openWeeklyReview(week)` → Tauri `open_current_review` / `open_weekly_review` → `PlanningApp::open_weekly_review`

## Steps

1. **Require ready store** — `require_database()` and `require_reports()` refuse if `StoreHealth` is not `Ready` or no sync folder is connected.
2. **Touch review record** — `touch_review(week)` upserts a `WeeklyReview` keyed by the week label; updates `last_opened_at`. Reopening never duplicates.
3. **Compute summary** — `weekly_summary(week)` queries current Tasks, Habits, Check-ins, and Goals for that Calendar Week. Nothing is read from a stored summary.
4. **Regenerate report** — `regenerate_report` writes `<sync>/weekly-reports/{week}-weekly-report.md`: replaces front matter and the summary comment region; preserves all other body bytes.
5. **Ensure next week's focus** — `weekly_focus(week.next())` loads or creates the coming week's `WeeklyFocus` record (empty if new).
6. **Read prior report** — `reports.read(week.previous())` returns the prior week's full body, if the file exists.
7. **Project view** — `build_review_view` assembles `WeeklyReviewView`: summary, reflection (body minus summary block), `previousReport`, `nextWeekFocus` task views, `reportPath`.

`open_current_review` resolves the current Calendar Week from the home-zone clock, then calls `open_weekly_review`.

## Reads

| Source | What |
|--------|------|
| `weekly_review` table | Existing review for the week key, if any |
| `task`, `habit`, `habit_check_in`, `goal` tables | Fresh `WeeklySummary` computation |
| `weekly_focus` table | Next week's focus (ensure step) |
| `<sync>/weekly-reports/{week}-weekly-report.md` | Current week's report body (reflection extraction) |
| `<sync>/weekly-reports/{prior-week}-weekly-report.md` | Prior week's full body, if present |

## Writes

| Target | When |
|--------|------|
| `weekly_review` table | Every open — upsert + `last_opened_at` |
| `weekly_focus` table | First access for `week.next()` only (create empty focus) |
| `weekly-reports/{week}-weekly-report.md` | Every open — front matter + summary region regenerated |

Reopening the same week writes the report file again (refreshed summary) but does not create a second file or duplicate review record.

## Side effects

- Creates `weekly-reports/` under the sync folder on first write.
- Creates one `.md` file per week reviewed (deterministic path).
- Ensures an empty Weekly Focus exists for the coming week.
- Prior week's report body is surfaced read-only in the view; it is not modified.

## Files to inspect

| Path | Role |
|------|------|
| `src-tauri/src/private/review_commands.rs` | IPC commands |
| `crates/planning-app/src/private/weekly_review_use_cases.rs` | `open_weekly_review`, `save_reflection`, `regenerate_report` |
| `crates/planning-app/src/private/weekly_summary.rs` | On-demand summary computation |
| `crates/planning-app/src/private/summary_markdown.rs` | Summary → Markdown |
| `crates/planning-reports/src/private/report_file.rs` | Path, read, write, `save_reflection` |
| `crates/planning-reports/src/private/summary_block.rs` | Marker-delimited region |
| `src/lib/api/index.ts` | Frontend `openWeeklyReview`, `openCurrentReview` |

## Common failure modes

| Symptom | Likely cause |
|---------|--------------|
| Command error "not ready" | Setup incomplete, sync conflict, or writer lock — `require_database` / `require_reports` refused |
| Report error on open | File has `schema` greater than `ReportFrontMatter::SCHEMA` — `UnsupportedSchema`, never silently overwritten |
| Summary region at top after user deleted markers | Expected — `SummaryBlock::replace` prepends a new block; user prose is preserved |
| Reflection shows summary headings | Markers corrupt or unterminated — whole body treated as user-owned; reflection editor shows full body |
| Prior report is `null` | No file for the previous Calendar Week yet |
| Summary changed after correcting old check-in | Expected — summaries are recomputed from current data, never frozen |
