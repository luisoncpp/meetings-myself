# Weekly Reviews & Report Files — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` (or subagent-driven
> development) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> Read [0001-self-planning-app.md](../0001-self-planning-app.md) first. Requires
> [0005-daily-plan-and-habits.md](0005-daily-plan-and-habits.md) to be complete. May run in
> parallel with [0007-ui-surfaces.md](../0007-ui-surfaces.md).

**Goal:** One canonical, externally editable Markdown Weekly Report per Calendar Week, with
always-current summaries and reflection preserved exactly as typed, plus the Weekly Review use
cases that read the prior week and prepare the coming one.

**Architecture:** `planning-reports` is a **domain-blind** crate: it knows about a Markdown file
with app-owned YAML front matter and one app-owned region delimited by HTML comments. It never
learns what a Task is. `planning-app` computes the summary from domain data, renders it to
Markdown, and hands the text over. That split is what lets the file format be tested exhaustively
without a database, and the summary logic be tested without touching the disk.

**The preservation contract — the single most important rule in this plan:**

> On every write, the front matter is replaced wholesale and the region between
> `<!-- self-planning:summary:start -->` and `<!-- self-planning:summary:end -->` is replaced
> wholesale. **Every other byte of the file is preserved exactly**, including whitespace, the
> user's own headings, and anything they added below, above, or between.

**Tech Stack:** As plan 0005, plus `serde_norway` 0.9 for YAML. `serde_yaml` is deprecated
(published as `0.9.34+deprecated`); `serde_norway` is the maintained, API-compatible fork.

---

## Global constraints

See [0001-self-planning-app.md](../0001-self-planning-app.md#global-constraints). Load-bearing here:

- **Weekly Report summaries are regenerated from current data, never frozen.** Reopening a
  three-week-old review shows corrected check-ins and reopened Tasks.
- **Only typed reflection is preserved verbatim.**
- **Reopening a past review never creates a second report file.** The filename is derived from the
  week, so there is exactly one path per week by construction.
- **The Weekly Review has no exclusive powers** — every action it offers exists in the Library too
  (acceptance criterion A8). This plan adds no method that only a review can call.

---

## File structure

| File | Responsibility |
|------|----------------|
| `crates/planning-reports/src/lib.rs` | Public interface |
| `crates/planning-reports/src/private/error.rs` | `ReportError` |
| `crates/planning-reports/src/private/front_matter.rs` | `ReportFrontMatter` parse + render |
| `crates/planning-reports/src/private/document.rs` | Split a file into front matter + body |
| `crates/planning-reports/src/private/summary_block.rs` | Marker-delimited region replacement |
| `crates/planning-reports/src/private/report_file.rs` | `WeeklyReportFile` — paths, read, write |
| `crates/planning-core/src/private/weekly_review.rs` | `WeeklyReview` entity |
| `crates/planning-app/src/private/weekly_summary.rs` | `WeeklySummary` computation |
| `crates/planning-app/src/private/summary_markdown.rs` | Summary → Markdown |
| `crates/planning-app/src/private/weekly_review_use_cases.rs` | Open, save reflection, prior report |
| `src-tauri/src/private/review_commands.rs` | Tauri commands |
| `docs/architecture/weekly-reports.md` | New architecture doc |
| `docs/flows/opening-a-weekly-review.md` | New flow doc |

---

### Task 1: The report document format

**Files:**
- Create: `crates/planning-reports/Cargo.toml`, `src/lib.rs`, `src/private/mod.rs`,
  `src/private/error.rs`, `src/private/front_matter.rs`, `src/private/document.rs`
- Modify: root `Cargo.toml`

**Interfaces:**
- Consumes: nothing from the other crates — this crate depends only on serde, chrono, and
  `serde_norway`.
- Produces:

```rust
pub struct ReportFrontMatter {
    pub week: String,            // "2026-W32"
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub schema: u32,             // ReportFrontMatter::SCHEMA == 1
    pub generated_at: DateTime<Utc>,
}
impl ReportFrontMatter { pub const SCHEMA: u32 = 1; pub fn render(&self) -> Result<String, ReportError>; }

pub struct ReportDocument { pub front_matter: ReportFrontMatter, pub body: String }
impl ReportDocument {
    pub fn parse(text: &str) -> Result<Self, ReportError>;
    pub fn render(&self) -> Result<String, ReportError>;
}

pub enum ReportError { Io, MissingFrontMatter, MalformedFrontMatter { detail: String },
                       UnsupportedSchema { found: u32 } }
```

Front matter is serialized `snake_case` (matching the Rust fields) so a human editing the file by
hand sees `week_start`, not `weekStart`. This differs from the IPC types on purpose: this file is
read by people, not by TypeScript.

- [ ] **Step 1: Create the crate**

Add `crates/planning-reports` to workspace members and
`planning-reports = { path = "crates/planning-reports" }` to `[workspace.dependencies]`.

```toml
[package]
name = "planning-reports"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
chrono = { workspace = true }
serde = { workspace = true }
serde_norway = "0.9.42"
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3.27.0"
```

- [ ] **Step 2: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const SAMPLE: &str = "---\n\
week: 2026-W32\n\
week_start: 2026-08-03\n\
week_end: 2026-08-09\n\
schema: 1\n\
generated_at: 2026-08-09T18:22:11Z\n\
---\n\
\n\
## Reflection\n\
\n\
It was a good week.\n";

    #[test]
    fn a_document_splits_into_front_matter_and_body() {
        let document = ReportDocument::parse(SAMPLE).unwrap();
        assert_eq!(document.front_matter.week, "2026-W32");
        assert_eq!(document.front_matter.week_start, NaiveDate::from_ymd_opt(2026, 8, 3).unwrap());
        assert_eq!(document.front_matter.schema, 1);
        assert_eq!(document.body, "\n## Reflection\n\nIt was a good week.\n");
    }

    #[test]
    fn rendering_a_parsed_document_reproduces_it_byte_for_byte() {
        let document = ReportDocument::parse(SAMPLE).unwrap();
        assert_eq!(document.render().unwrap(), SAMPLE);
    }

    #[test]
    fn a_body_containing_a_horizontal_rule_is_not_mistaken_for_a_delimiter() {
        let text = format!("{SAMPLE}\n---\n\nMore prose after a rule.\n");
        let document = ReportDocument::parse(&text).unwrap();
        assert!(document.body.contains("More prose after a rule."));
        assert_eq!(document.front_matter.week, "2026-W32");
    }

    #[test]
    fn a_file_without_front_matter_is_reported_not_guessed_at() {
        assert!(matches!(
            ReportDocument::parse("Just some notes.\n").unwrap_err(),
            ReportError::MissingFrontMatter
        ));
    }

    #[test]
    fn an_unknown_schema_is_refused_rather_than_silently_overwritten() {
        let text = SAMPLE.replace("schema: 1", "schema: 99");
        assert!(matches!(
            ReportDocument::parse(&text).unwrap_err(),
            ReportError::UnsupportedSchema { found: 99 }
        ));
    }
}
```

The horizontal-rule test is the one that matters: a naive `split("---")` corrupts any report where
the user typed a Markdown rule. Only the **first two** delimiters may be consumed.

- [ ] **Step 3: Run to verify it fails**

```bash
cargo test -p planning-reports
```

Expected: FAIL — `cannot find struct 'ReportDocument'`.

- [ ] **Step 4: Implement `error.rs` and `front_matter.rs`**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("could not read or write the report file")]
    Io(#[from] std::io::Error),

    #[error("the file has no YAML front matter block")]
    MissingFrontMatter,

    #[error("the front matter is not valid: {detail}")]
    MalformedFrontMatter { detail: String },

    #[error("this report uses schema {found}, which this version does not understand")]
    UnsupportedSchema { found: u32 },
}
```

```rust
use super::error::ReportError;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// App-owned metadata. Written in snake_case because a human edits this file by
/// hand — unlike the IPC types, which are camelCase for TypeScript.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFrontMatter {
    pub week: String,
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub schema: u32,
    pub generated_at: DateTime<Utc>,
}

impl ReportFrontMatter {
    pub const SCHEMA: u32 = 1;

    pub fn render(&self) -> Result<String, ReportError> {
        serde_norway::to_string(self)
            .map_err(|error| ReportError::MalformedFrontMatter { detail: error.to_string() })
    }

    pub fn parse(yaml: &str) -> Result<Self, ReportError> {
        let parsed: Self = serde_norway::from_str(yaml)
            .map_err(|error| ReportError::MalformedFrontMatter { detail: error.to_string() })?;
        if parsed.schema > Self::SCHEMA {
            return Err(ReportError::UnsupportedSchema { found: parsed.schema });
        }
        Ok(parsed)
    }
}
```

- [ ] **Step 5: Implement `document.rs`**

```rust
use super::error::ReportError;
use super::front_matter::ReportFrontMatter;

const DELIMITER: &str = "---";

pub struct ReportDocument {
    pub front_matter: ReportFrontMatter,
    pub body: String,
}

impl ReportDocument {
    /// Consumes exactly the first two delimiters. A `---` horizontal rule later in
    /// the body is ordinary prose and must survive untouched.
    pub fn parse(text: &str) -> Result<Self, ReportError> {
        let rest = text
            .strip_prefix(DELIMITER)
            .and_then(|rest| rest.strip_prefix('\n'))
            .ok_or(ReportError::MissingFrontMatter)?;

        let (yaml, body) = split_at_closing_delimiter(rest).ok_or(ReportError::MissingFrontMatter)?;
        Ok(Self { front_matter: ReportFrontMatter::parse(yaml)?, body: body.to_string() })
    }

    pub fn render(&self) -> Result<String, ReportError> {
        let yaml = self.front_matter.render()?;
        Ok(format!("{DELIMITER}\n{yaml}{DELIMITER}\n{}", self.body))
    }
}

/// Finds the closing `---` that stands alone on its own line.
fn split_at_closing_delimiter(rest: &str) -> Option<(&str, &str)> {
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == DELIMITER {
            let body_start = offset + line.len();
            return Some((&rest[..offset], &rest[body_start..]));
        }
        offset += line.len();
    }
    None
}
```

`serde_norway::to_string` already emits a trailing newline, which is why `render` does not add one
before the closing delimiter. If the round-trip test fails on whitespace, adjust `render` rather
than loosening the test — byte-for-byte round-tripping is the contract.

- [ ] **Step 6: Run, export, commit**

```bash
cargo test -p planning-reports
```

Expected: PASS — 5 tests.

```bash
git add Cargo.toml Cargo.lock crates/planning-reports
git commit -m "feat(reports): add weekly report document parsing with byte-exact round-trip"
```

---

### Task 2: The app-owned summary region

**Files:**
- Create: `crates/planning-reports/src/private/summary_block.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:
  - `SummaryBlock::START: &str = "<!-- self-planning:summary:start -->"`
  - `SummaryBlock::END: &str = "<!-- self-planning:summary:end -->"`
  - `SummaryBlock::replace(body: &str, markdown: &str) -> String` — swaps the region's contents,
    preserving everything outside it exactly; re-inserts the block at the top if the markers are
    missing.
  - `SummaryBlock::reflection(body: &str) -> String` — the body with the block removed, which is
    what the UI shows in the reflection editor.

**This is the preservation contract's implementation.** Every test here describes a way a user
could lose writing, which is the one failure this app must never have.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn body_with_block() -> String {
        format!(
            "{}\n## Week in review\n\nOld numbers.\n{}\n\n## Reflection\n\nMy own words.\n",
            SummaryBlock::START,
            SummaryBlock::END
        )
    }

    #[test]
    fn replacing_the_summary_leaves_every_other_byte_alone() {
        let updated = SummaryBlock::replace(&body_with_block(), "## Week in review\n\nNew numbers.");
        assert!(updated.contains("New numbers."));
        assert!(!updated.contains("Old numbers."));
        assert!(
            updated.ends_with("\n## Reflection\n\nMy own words.\n"),
            "the reflection must survive exactly: {updated:?}"
        );
    }

    #[test]
    fn replacing_twice_is_stable() {
        let once = SummaryBlock::replace(&body_with_block(), "A");
        let twice = SummaryBlock::replace(&once, "A");
        assert_eq!(once, twice, "regeneration must not accumulate markers or blank lines");
    }

    #[test]
    fn text_the_user_wrote_above_the_block_survives() {
        let body = format!("My preamble.\n\n{}\nold\n{}\n", SummaryBlock::START, SummaryBlock::END);
        let updated = SummaryBlock::replace(&body, "new");
        assert!(updated.starts_with("My preamble.\n\n"));
        assert!(updated.contains("new"));
    }

    #[test]
    fn a_body_whose_markers_were_deleted_gets_them_back_without_losing_prose() {
        let body = "## Reflection\n\nI deleted the app's block.\n";
        let updated = SummaryBlock::replace(body, "regenerated");
        assert!(updated.contains("regenerated"));
        assert!(
            updated.contains("I deleted the app's block."),
            "restoring the block must never cost the user a word"
        );
        // And it is stable from then on.
        assert_eq!(SummaryBlock::replace(&updated, "regenerated"), updated);
    }

    #[test]
    fn the_reflection_is_the_body_without_the_block() {
        let reflection = SummaryBlock::reflection(&body_with_block());
        assert!(!reflection.contains("Old numbers."));
        assert!(!reflection.contains(SummaryBlock::START));
        assert_eq!(reflection.trim(), "## Reflection\n\nMy own words.");
    }

    #[test]
    fn an_unterminated_start_marker_does_not_swallow_the_rest_of_the_file() {
        let body = format!("{}\nunclosed\n\n## Reflection\n\nMine.\n", SummaryBlock::START);
        let updated = SummaryBlock::replace(&body, "new");
        assert!(updated.contains("Mine."), "a corrupt marker must not delete prose");
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-reports summary_block
```

```rust
/// The app owns exactly one region of the report body. Everything outside it
/// belongs to the user and is preserved byte for byte.
pub struct SummaryBlock;

impl SummaryBlock {
    pub const START: &'static str = "<!-- self-planning:summary:start -->";
    pub const END: &'static str = "<!-- self-planning:summary:end -->";

    pub fn replace(body: &str, markdown: &str) -> String {
        let block = format!("{}\n{}\n{}\n", Self::START, markdown.trim_end(), Self::END);
        let Some((before, after)) = Self::split(body) else {
            // No usable block: prepend one. Never rewrite or drop existing prose.
            return format!("{block}\n{body}");
        };
        format!("{before}{block}{after}")
    }

    pub fn reflection(body: &str) -> String {
        let Some((before, after)) = Self::split(body) else {
            return body.to_string();
        };
        format!("{before}{after}")
    }

    /// Splits into (text before the block, text after it). Returns None when the
    /// markers are absent or the start marker is unterminated — in both cases the
    /// safe move is to treat the whole body as the user's.
    fn split(body: &str) -> Option<(&str, &str)> {
        let start = body.find(Self::START)?;
        let end_marker = body[start..].find(Self::END)? + start;
        let after = end_marker + Self::END.len();
        let after = body[after..].strip_prefix('\n').map_or(after, |_| after + 1);
        Some((&body[..start], &body[after..]))
    }
}
```

`replace` must be idempotent, which is why the block is built with exactly one trailing newline and
`split` consumes the newline after the end marker. If the "replacing twice is stable" test fails,
fix the newline handling here rather than relaxing the assertion.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-reports
```

Expected: PASS — 11 tests.

```bash
git add crates/planning-reports
git commit -m "feat(reports): add the app-owned summary region with a preservation contract"
```

---

### Task 3: `WeeklyReportFile`

**Files:**
- Create: `crates/planning-reports/src/private/report_file.rs`
- Modify: `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:
  - `WeeklyReportFile::at(sync_folder: PathBuf) -> Self`
  - `WeeklyReportFile::FOLDER: &str = "weekly-reports"`
  - `path_for(&self, week_label: &str) -> PathBuf` — `<sync>/weekly-reports/2026-W32-weekly-report.md`
  - `read(&self, week_label: &str) -> Result<Option<ReportDocument>, ReportError>`
  - `write(&self, WriteReport { front_matter, summary_markdown }) -> Result<(), ReportError>` —
    reads the existing file if present, replaces its summary region, writes back.
  - `save_reflection(&self, SaveBody { week_label, reflection }) -> Result<(), ReportError>` —
    replaces the non-summary part of the body, keeping the summary region.

The deterministic filename **is** the "exactly one report per week" guarantee — the same mechanism
as the record keys in plan 0005.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn front_matter() -> ReportFrontMatter {
        ReportFrontMatter {
            week: "2026-W32".into(),
            week_start: NaiveDate::from_ymd_opt(2026, 8, 3).unwrap(),
            week_end: NaiveDate::from_ymd_opt(2026, 8, 9).unwrap(),
            schema: ReportFrontMatter::SCHEMA,
            generated_at: Utc.with_ymd_and_hms(2026, 8, 9, 18, 0, 0).unwrap(),
        }
    }

    #[test]
    fn the_filename_is_derived_from_the_week_so_there_is_one_per_week() {
        let folder = TempDir::new().unwrap();
        let reports = WeeklyReportFile::at(folder.path().to_path_buf());
        assert!(reports
            .path_for("2026-W32")
            .ends_with("weekly-reports/2026-W32-weekly-report.md"));
    }

    #[test]
    fn writing_twice_updates_one_file_and_preserves_the_reflection() {
        let folder = TempDir::new().unwrap();
        let reports = WeeklyReportFile::at(folder.path().to_path_buf());

        reports
            .write(WriteReport { front_matter: front_matter(), summary_markdown: "Old".into() })
            .unwrap();
        reports
            .save_reflection(SaveBody {
                week_label: "2026-W32".into(),
                reflection: "## Reflection\n\nI learned a lot.\n".into(),
            })
            .unwrap();

        // Regenerating the summary later must not touch the reflection (ADR 0002).
        reports
            .write(WriteReport { front_matter: front_matter(), summary_markdown: "New".into() })
            .unwrap();

        let document = reports.read("2026-W32").unwrap().unwrap();
        assert!(document.body.contains("New"));
        assert!(!document.body.contains("Old"));
        assert!(document.body.contains("I learned a lot."));

        let files: Vec<_> =
            std::fs::read_dir(folder.path().join(WeeklyReportFile::FOLDER)).unwrap().collect();
        assert_eq!(files.len(), 1, "A4: exactly one report file per week");
    }

    #[test]
    fn reading_a_week_that_has_no_report_yet_is_none_rather_than_an_error() {
        let folder = TempDir::new().unwrap();
        let reports = WeeklyReportFile::at(folder.path().to_path_buf());
        assert!(reports.read("2026-W31").unwrap().is_none());
    }

    #[test]
    fn a_report_edited_externally_is_read_back_with_the_edit_intact() {
        let folder = TempDir::new().unwrap();
        let reports = WeeklyReportFile::at(folder.path().to_path_buf());
        reports
            .write(WriteReport { front_matter: front_matter(), summary_markdown: "S".into() })
            .unwrap();

        // Simulate the user editing the file in another editor while the app is closed.
        let path = reports.path_for("2026-W32");
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}\n## My own section\n\nAdded elsewhere.\n")).unwrap();

        reports
            .write(WriteReport { front_matter: front_matter(), summary_markdown: "S2".into() })
            .unwrap();
        let document = reports.read("2026-W32").unwrap().unwrap();
        assert!(document.body.contains("Added elsewhere."));
        assert!(document.body.contains("S2"));
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-reports report_file
```

```rust
use super::document::ReportDocument;
use super::error::ReportError;
use super::front_matter::ReportFrontMatter;
use super::summary_block::SummaryBlock;
use std::path::{Path, PathBuf};

pub struct WriteReport {
    pub front_matter: ReportFrontMatter,
    pub summary_markdown: String,
}

pub struct SaveBody {
    pub week_label: String,
    pub reflection: String,
}

/// One Markdown file per Calendar Week inside the Synchronization Folder. The
/// deterministic name is what guarantees a week can never gain a second report.
pub struct WeeklyReportFile {
    root: PathBuf,
}

impl WeeklyReportFile {
    pub const FOLDER: &'static str = "weekly-reports";

    pub fn at(sync_folder: PathBuf) -> Self {
        Self { root: sync_folder }
    }

    pub fn path_for(&self, week_label: &str) -> PathBuf {
        self.root.join(Self::FOLDER).join(format!("{week_label}-weekly-report.md"))
    }

    pub fn read(&self, week_label: &str) -> Result<Option<ReportDocument>, ReportError> {
        let path = self.path_for(week_label);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(ReportDocument::parse(&std::fs::read_to_string(path)?)?))
    }

    /// Regenerates the summary against the current data, preserving everything the
    /// user wrote. Creates the file with a starter body on first write.
    pub fn write(&self, request: WriteReport) -> Result<(), ReportError> {
        let week_label = request.front_matter.week.clone();
        let existing_body = self.read(&week_label)?.map(|document| document.body);
        let body = existing_body.unwrap_or_else(starter_body);

        let document = ReportDocument {
            front_matter: request.front_matter,
            body: SummaryBlock::replace(&body, &request.summary_markdown),
        };
        self.save(&week_label, &document.render()?)
    }

    /// Replaces the user's part of the body, keeping the app's summary region.
    pub fn save_reflection(&self, request: SaveBody) -> Result<(), ReportError> {
        let mut document =
            self.read(&request.week_label)?.ok_or(ReportError::MissingFrontMatter)?;
        let summary = SummaryBlock::extract(&document.body).unwrap_or_default();
        document.body = SummaryBlock::replace(&request.reflection, &summary);
        self.save(&request.week_label, &document.render()?)
    }

    fn save(&self, week_label: &str, text: &str) -> Result<(), ReportError> {
        let path = self.path_for(week_label);
        create_parent(&path)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

fn starter_body() -> String {
    "\n\n## Reflection\n\n".to_string()
}

fn create_parent(path: &Path) -> Result<(), ReportError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    Ok(())
}
```

`save_reflection` needs `SummaryBlock::extract(body: &str) -> Option<String>` returning the region's
current contents. Add it beside `split` in Task 2's file, with a test.

- [ ] **Step 3: Run, export everything from `lib.rs`, commit**

```rust
//! Weekly Report files: app-owned front matter and one app-owned body region
//! inside an otherwise user-owned Markdown document. This crate knows nothing
//! about Tasks, Habits, or the database.

mod private;

pub use private::document::ReportDocument;
pub use private::error::ReportError;
pub use private::front_matter::ReportFrontMatter;
pub use private::report_file::{SaveBody, WeeklyReportFile, WriteReport};
pub use private::summary_block::SummaryBlock;
```

```bash
cargo test -p planning-reports
```

Expected: PASS.

```bash
git add crates/planning-reports
git commit -m "feat(reports): add deterministic per-week report files"
```

---

### Task 4: The weekly summary

**Files:**
- Create: `crates/planning-core/src/private/weekly_review.rs`,
  `crates/planning-app/src/private/weekly_summary.rs`,
  `crates/planning-app/src/private/summary_markdown.rs`
- Modify: `crates/planning-app/Cargo.toml` (add `planning-reports`), `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:

```rust
// planning-core
pub struct WeeklyReview { pub id: WeeklyReviewId, pub week: CalendarWeek,
                          pub created_at: DateTime<Utc>, pub last_opened_at: DateTime<Utc> }
impl WeeklyReview { pub fn key(week: CalendarWeek) -> String; pub fn start(StartReview { week, clock }) -> Self; }

// planning-app
pub struct HabitSummary { pub title: String, pub done: u32, pub skipped: u32, pub not_completed: u32 }
pub struct WeeklySummary {
    pub week: CalendarWeek,
    pub completed: Vec<String>,     // titles, in completion order
    pub still_open: usize,
    pub overdue: Vec<String>,
    pub habits: Vec<HabitSummary>,
    pub goals_achieved: Vec<String>,
}
PlanningApp::weekly_summary(&self, CalendarWeek) -> Result<WeeklySummary, AppError>
summary_markdown::render(&WeeklySummary) -> String
```

The summary is **computed on demand from current data every time** — nothing about it is stored.
That is what makes "corrected a check-in three weeks later" show up in the old report without a
migration.

**Tone constraint (PRODUCT.md, DESIGN.md):** the rendered Markdown contains no streaks, no
percentages, no scores, and no evaluative language. Counts and titles only. A test asserts the
absence of `%`, `streak`, and `score`.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn the_summary_reflects_current_data_including_later_corrections() {
    let (_home, _drive, app, clock) = app_on(7).await;   // Friday 2026-08-07, week 2026-W32
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let today = app.calendar().unwrap().today(app.clock_ref());

    let done = app.create_task("Prepare portfolio".into()).await.unwrap();
    app.complete_task(&done.id).await.unwrap();
    let open = app.create_task("Call the bank".into()).await.unwrap();
    app.set_task_deadline(SetDeadline {
        task: &open.id,
        deadline: Some(today - Duration::days(1)),
    })
    .await
    .unwrap();

    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: today, outcome: CheckInOutcome::NotCompleted,
    })
    .await
    .unwrap();

    let before = app.weekly_summary(week).await.unwrap();
    assert_eq!(before.completed, vec!["Prepare portfolio".to_string()]);
    assert_eq!(before.overdue, vec!["Call the bank".to_string()]);
    assert_eq!(before.habits[0].not_completed, 1);

    // Two weeks later, correct the past check-in.
    clock.advance(Duration::days(14));
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: today, outcome: CheckInOutcome::Done,
    })
    .await
    .unwrap();

    let after = app.weekly_summary(week).await.unwrap();
    assert_eq!(after.habits[0].done, 1, "summaries are never frozen (ADR 0002)");
    assert_eq!(after.habits[0].not_completed, 0);
}

#[tokio::test]
async fn the_summary_counts_only_the_weeks_own_days() {
    let (_home, _drive, app, clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let habit = app
        .create_habit(NewHabit { title: "Writing".into(), cadence: Cadence::EveryDay })
        .await
        .unwrap();
    let today = app.calendar().unwrap().today(app.clock_ref());

    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: today, outcome: CheckInOutcome::Done,
    })
    .await
    .unwrap();
    // Monday of the NEXT week.
    app.record_check_in(CheckInRequest {
        habit: habit.id.clone(), date: week.next().monday(), outcome: CheckInOutcome::Done,
    })
    .await
    .unwrap();

    assert_eq!(app.weekly_summary(week).await.unwrap().habits[0].done, 1);
    assert_eq!(app.weekly_summary(week.next()).await.unwrap().habits[0].done, 1);
}

#[test]
fn the_rendered_summary_never_scores_or_gamifies() {
    let summary = WeeklySummary {
        week: CalendarWeek::containing(NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()),
        completed: vec!["Prepare portfolio".into()],
        still_open: 3,
        overdue: vec!["Call the bank".into()],
        habits: vec![HabitSummary {
            title: "Writing".into(), done: 4, skipped: 1, not_completed: 2,
        }],
        goals_achieved: vec![],
    };

    let markdown = summary_markdown::render(&summary);
    assert!(markdown.contains("Prepare portfolio"));
    assert!(markdown.contains("Writing"));
    for banned in ["%", "streak", "Streak", "score", "Score", "🔥"] {
        assert!(!markdown.contains(banned), "PRODUCT.md forbids {banned} in reports");
    }
}
```

- [ ] **Step 2: Run to verify it fails, then implement `weekly_summary.rs`**

```bash
cargo test -p planning-app weekly_summary
```

```rust
use super::error::AppError;
use super::service::PlanningApp;
use super::check_in_use_cases::DateRange;
use planning_core::{CalendarWeek, CheckInOutcome, Completion, Habit, HabitId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitSummary {
    pub title: String,
    pub done: u32,
    pub skipped: u32,
    pub not_completed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummary {
    pub week: CalendarWeek,
    pub completed: Vec<String>,
    pub still_open: usize,
    pub overdue: Vec<String>,
    pub habits: Vec<HabitSummary>,
    pub goals_achieved: Vec<String>,
}

impl PlanningApp {
    /// Computed fresh on every call. Nothing here is stored, which is why a
    /// correction made weeks later appears in an old report (ADR 0002).
    pub async fn weekly_summary(&self, week: CalendarWeek) -> Result<WeeklySummary, AppError> {
        let today = self.calendar()?.today(self.clock.as_ref());
        let tasks = self.tasks().await?;

        let completed = tasks
            .iter()
            .filter(|task| matches!(task.completion, Completion::Completed { on } if week.contains(on)))
            .map(|task| task.title.clone())
            .collect();

        let overdue = tasks
            .iter()
            .filter(|task| task.is_overdue(today))
            .map(|task| task.title.clone())
            .collect();

        let still_open = tasks
            .iter()
            .filter(|task| task.lifecycle.is_active() && !task.completion.is_complete())
            .count();

        Ok(WeeklySummary {
            week,
            completed,
            still_open,
            overdue,
            habits: self.summarize_habits(week).await?,
            goals_achieved: self.goals_achieved_in(week).await?,
        })
    }

    async fn summarize_habits(&self, week: CalendarWeek) -> Result<Vec<HabitSummary>, AppError> {
        let check_ins = self
            .check_ins_between(DateRange { from: week.monday(), to: week.sunday() })
            .await?;
        let mut tallies: HashMap<HabitId, (u32, u32, u32)> = HashMap::new();
        for check_in in check_ins {
            let entry = tallies.entry(check_in.habit).or_default();
            match check_in.outcome {
                CheckInOutcome::Done => entry.0 += 1,
                CheckInOutcome::Skipped => entry.1 += 1,
                CheckInOutcome::NotCompleted => entry.2 += 1,
            }
        }
        Ok(self
            .habits()
            .await?
            .into_iter()
            .filter_map(|habit| tallies.get(&habit.id).map(|counts| summarize(&habit, *counts)))
            .collect())
    }
}

fn summarize(habit: &Habit, counts: (u32, u32, u32)) -> HabitSummary {
    HabitSummary {
        title: habit.title.clone(),
        done: counts.0,
        skipped: counts.1,
        not_completed: counts.2,
    }
}
```

`weekly_summary` exceeds 30 lines — split `completed`, `overdue`, and `still_open` into three small
private functions taking `(&[Task], week)` or `(&[Task], today)`. Add `goals_achieved_in` following
the same shape as `summarize_habits`.

- [ ] **Step 3: Implement `summary_markdown.rs`**

```rust
use super::weekly_summary::{HabitSummary, WeeklySummary};

/// Renders the app-owned region of a Weekly Report.
///
/// Counts and titles only: no percentages, no streaks, no evaluative language.
/// PRODUCT.md is explicit that reflection must not become scoring.
pub fn render(summary: &WeeklySummary) -> String {
    let mut out = String::from("## Week in review\n\n");
    out.push_str(&section("Completed", &summary.completed));
    out.push_str(&section("Overdue", &summary.overdue));
    out.push_str(&section("Goals achieved", &summary.goals_achieved));
    out.push_str(&format!("**Still open:** {}\n\n", summary.still_open));
    out.push_str(&habits(&summary.habits));
    out
}

fn section(heading: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("**{heading}:** none\n\n");
    }
    let lines: Vec<String> = items.iter().map(|item| format!("- {item}")).collect();
    format!("**{heading}:**\n\n{}\n\n", lines.join("\n"))
}

fn habits(entries: &[HabitSummary]) -> String {
    if entries.is_empty() {
        return "**Habits:** no check-ins recorded\n".to_string();
    }
    let mut out = String::from("**Habits:**\n\n| Habit | Done | Skipped | Not completed |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for entry in entries {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            entry.title, entry.done, entry.skipped, entry.not_completed
        ));
    }
    out
}
```

- [ ] **Step 4: Run, commit**

```bash
cargo test -p planning-app weekly_summary
```

Expected: PASS — 3 tests.

```bash
git add crates/planning-core crates/planning-app
git commit -m "feat(app): compute weekly summaries fresh from current data"
```

---

### Task 5: Weekly Review use cases

**Files:**
- Create: `crates/planning-app/src/private/weekly_review_use_cases.rs`
- Modify: `private/service.rs` (hold a `WeeklyReportFile`), `private/mod.rs`, `lib.rs`

**Interfaces:**
- Produces:

```rust
pub struct WeeklyReviewView {
    pub week: CalendarWeek,
    pub summary: WeeklySummary,
    pub reflection: String,
    pub previous_report: Option<String>,   // the prior week's full body
    pub next_week_focus: Vec<TaskView>,
    pub report_path: PathBuf,
}

PlanningApp::open_weekly_review(&self, CalendarWeek) -> Result<WeeklyReviewView, AppError>
PlanningApp::open_current_review(&self) -> Result<WeeklyReviewView, AppError>
PlanningApp::save_reflection(&self, SaveReflection { week, reflection }) -> Result<(), AppError>
PlanningApp::report_path(&self, CalendarWeek) -> Result<PathBuf, AppError>
```

`open_weekly_review` does four things and nothing else:
1. upserts the `WeeklyReview` record (key = week label, so reopening never duplicates);
2. regenerates the report file's summary region from current data;
3. ensures a Weekly Focus exists for `week.next()`;
4. reads the prior week's report body, if any.

**This task claims acceptance criterion A4.**

`service.rs` gains `pub(crate) reports: Option<WeeklyReportFile>`, set in `reconnect` whenever a
sync folder is present, so report paths always sit beside the database.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn opening_a_review_creates_one_report_prepares_next_week_and_shows_the_prior_one() {
    let (_home, drive, app, clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());

    let task = app.create_task("Prepare portfolio".into()).await.unwrap();
    app.complete_task(&task.id).await.unwrap();

    let view = app.open_weekly_review(week).await.unwrap();
    assert_eq!(view.week, week);
    assert!(view.summary.completed.contains(&"Prepare portfolio".to_string()));
    assert_eq!(view.previous_report, None, "there is no earlier week yet");
    assert!(view.report_path.exists());

    // A Weekly Focus for the coming week now exists and is adjustable.
    let next = app.weekly_focus(week.next()).await.unwrap();
    assert!(next.tasks.is_empty());

    // Move to the next week and review again.
    clock.advance(Duration::days(7));
    let later = app.calendar().unwrap().current_week(app.clock_ref());
    let second = app.open_weekly_review(later).await.unwrap();
    assert!(
        second.previous_report.unwrap().contains("Prepare portfolio"),
        "A4: the review shows the prior report"
    );

    let files: Vec<_> = std::fs::read_dir(drive.path().join("weekly-reports")).unwrap().collect();
    assert_eq!(files.len(), 2, "one file per week, never more");
}

#[tokio::test]
async fn reopening_a_past_review_refreshes_it_without_creating_a_duplicate() {
    let (_home, drive, app, clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());

    app.open_weekly_review(week).await.unwrap();
    app.save_reflection(SaveReflection {
        week,
        reflection: "## Reflection\n\nA quiet week.\n".into(),
    })
    .await
    .unwrap();

    // Three weeks later, complete a Task dated inside that week and reopen.
    clock.advance(Duration::days(21));
    let task = app.create_task("Late entry".into()).await.unwrap();
    app.complete_task(&task.id).await.unwrap();

    let reopened = app.open_weekly_review(week).await.unwrap();
    assert!(reopened.reflection.contains("A quiet week."), "reflection is preserved as written");
    assert!(
        !reopened.summary.completed.contains(&"Late entry".to_string()),
        "the Task was completed in a later week, so it belongs to that week's summary"
    );

    let files: Vec<_> = std::fs::read_dir(drive.path().join("weekly-reports")).unwrap().collect();
    assert_eq!(files.len(), 1, "A4: reopening never creates a second report");
}

#[tokio::test]
async fn every_review_action_is_also_available_without_a_review() {
    // A8: the Weekly Review has no exclusive powers.
    let (_home, _drive, app, _clock) = app_on(7).await;
    let week = app.calendar().unwrap().current_week(app.clock_ref());
    let goal = app
        .create_goal(NewGoal { title: "Career transition".into(), target_date: None })
        .await
        .unwrap();
    let task = app.create_task("Prepare portfolio".into()).await.unwrap();

    // No review has been opened, yet both actions succeed.
    app.achieve_goal(&goal.id).await.unwrap();
    app.add_to_focus(FocusChange { week: week.next(), task: task.id }).await.unwrap();

    assert!(app.goal(&goal.id).await.unwrap().unwrap().achievement.is_achieved());
    assert_eq!(app.weekly_focus(week.next()).await.unwrap().tasks.len(), 1);
}
```

- [ ] **Step 2: Run to verify it fails, then implement**

```bash
cargo test -p planning-app weekly_review
```

```rust
use super::error::AppError;
use super::service::PlanningApp;
use super::summary_markdown;
use super::weekly_summary::WeeklySummary;
use planning_core::{CalendarWeek, StartReview, WeeklyReview, WeeklyReviewId};
use planning_reports::{ReportFrontMatter, SaveBody, WriteReport};
use std::path::PathBuf;

pub struct SaveReflection {
    pub week: CalendarWeek,
    pub reflection: String,
}

impl PlanningApp {
    pub async fn open_current_review(&self) -> Result<WeeklyReviewView, AppError> {
        let week = self.calendar()?.current_week(self.clock.as_ref());
        self.open_weekly_review(week).await
    }

    /// Idempotent by construction: the review record's key is the week label and
    /// the report's filename is derived from it, so reopening refreshes rather
    /// than duplicating (A4).
    pub async fn open_weekly_review(
        &self,
        week: CalendarWeek,
    ) -> Result<WeeklyReviewView, AppError> {
        self.touch_review(week).await?;
        let summary = self.weekly_summary(week).await?;
        self.regenerate_report(&summary).await?;
        // The Weekly Review prepares the coming week; creating the focus here is
        // just an ensure — the Library can do the same thing at any time (A8).
        self.weekly_focus(week.next()).await?;

        let reports = self.require_reports()?;
        let document = reports.read(&week.label())?;
        Ok(WeeklyReviewView {
            week,
            summary,
            reflection: document
                .map(|found| planning_reports::SummaryBlock::reflection(&found.body))
                .unwrap_or_default(),
            previous_report: reports
                .read(&week.previous().label())?
                .map(|found| found.body),
            next_week_focus: self.focus_task_views(week.next()).await?,
            report_path: reports.path_for(&week.label()),
        })
    }

    pub async fn save_reflection(&self, request: SaveReflection) -> Result<(), AppError> {
        // Make sure the file exists before writing into it.
        let summary = self.weekly_summary(request.week).await?;
        self.regenerate_report(&summary).await?;
        self.require_reports()?.save_reflection(SaveBody {
            week_label: request.week.label(),
            reflection: request.reflection,
        })?;
        Ok(())
    }

    async fn regenerate_report(&self, summary: &WeeklySummary) -> Result<(), AppError> {
        let week = summary.week;
        self.require_reports()?.write(WriteReport {
            front_matter: ReportFrontMatter {
                week: week.label(),
                week_start: week.monday(),
                week_end: week.sunday(),
                schema: ReportFrontMatter::SCHEMA,
                generated_at: self.clock.now(),
            },
            summary_markdown: summary_markdown::render(summary),
        })?;
        Ok(())
    }

    async fn touch_review(&self, week: CalendarWeek) -> Result<(), AppError> {
        let key = WeeklyReview::key(week);
        let existing: Option<WeeklyReview> = self.load_one(WeeklyReviewId::TABLE, &key).await?;
        let mut review = existing
            .unwrap_or_else(|| WeeklyReview::start(StartReview { week, clock: self.clock.as_ref() }));
        review.last_opened_at = self.clock.now();
        self.store(WeeklyReviewId::TABLE, &key, &review).await?;
        Ok(())
    }
}
```

Add `require_reports(&self) -> Result<&WeeklyReportFile, AppError>` to `service.rs`, returning
`Err(AppError::NotReady(..))` unless health is `Ready`, and add
`#[error(transparent)] Report(#[from] ReportError)` to `AppError`. `focus_task_views` maps the
focus's task ids through `TaskView::project`.

- [ ] **Step 3: Run, commit**

```bash
cargo test -p planning-app
```

Expected: PASS. **A4 and A8 are now proven at the API level.**

```bash
git add crates/planning-app crates/planning-core
git commit -m "feat(app): add Weekly Review with regenerated summaries and preserved reflection"
```

---

### Task 6: Tauri commands and the TypeScript mirror

**Files:**
- Create: `src-tauri/src/private/review_commands.rs`
- Modify: `src-tauri/src/lib.rs`, `src/lib/domain/index.ts`, `src/lib/api/index.ts`
- Test: `src/lib/api/review.test.ts`, JSON-shape tests in `weekly_summary.rs`

| Command | Args | Returns |
|---------|------|---------|
| `open_weekly_review` | `week: string` | `WeeklyReviewView` |
| `open_current_review` | — | `WeeklyReviewView` |
| `save_reflection` | `week`, `reflection` | `void` |
| `weekly_summary` | `week` | `WeeklySummary` |
| `report_path` | `week` | `string` |

- [ ] **Step 1: Add JSON-shape tests, then the commands and the mirror**

```ts
export interface HabitSummary {
  title: string;
  done: number;
  skipped: number;
  notCompleted: number;
}

export interface WeeklySummary {
  week: string;
  completed: string[];
  stillOpen: number;
  overdue: string[];
  habits: HabitSummary[];
  goalsAchieved: string[];
}

export interface WeeklyReviewView {
  week: string;
  summary: WeeklySummary;
  reflection: string;
  previousReport: string | null;
  nextWeekFocus: TaskView[];
  reportPath: string;
}
```

`report_path` exists so the UI can offer "open this file in your editor" via
`tauri-plugin-opener` — the report is meant to be externally editable, and the app should say so.

- [ ] **Step 2: Run the gate and commit**

```bash
npm run check && fallow audit
```

```bash
git add src-tauri src/lib
git commit -m "feat: expose the Weekly Review API to the frontend"
```

---

### Task 7: Documentation

**Files:**
- Create: `docs/architecture/weekly-reports.md`, `docs/flows/opening-a-weekly-review.md`,
  `docs/lessons-learned/app-owned-regions-in-user-owned-files.md`
- Modify: the three README index tables, `docs/live/current-status.md`

- [ ] **Step 1: Write `docs/architecture/weekly-reports.md`** (target 80 lines)

Include a complete annotated example file. Cover: the exact filename pattern and why it is the
one-per-week guarantee; front matter fields and that they are snake_case for human readers;
the two markers and the preservation contract stated verbatim; that summaries are recomputed on
every open and never stored; that `planning-reports` is domain-blind; and the `schema` field's
forward-compatibility role (a newer schema is refused, never silently overwritten).

- [ ] **Step 2: Write `docs/flows/opening-a-weekly-review.md`**

Trigger (user opens the Weekly Review window) → `open_weekly_review` → touch review record →
compute summary → regenerate the report's summary region → ensure next week's focus → read the
prior week's report → project the view. Reads / Writes / Side effects (creates
`weekly-reports/` and one `.md` file) / Failure modes (store not `Ready`, a report file with a
newer `schema`, a report whose markers the user deleted).

- [ ] **Step 3: Write `docs/lessons-learned/app-owned-regions-in-user-owned-files.md`**

Topic: how to co-own a plain-text file with a human. The pattern: the app owns front matter plus
one comment-delimited region and treats every other byte as sacred; regeneration must be
idempotent; a missing or corrupt marker means "the whole body is theirs", never "rewrite it". The
counter-intuitive part: the dangerous operation is not writing, it is *parsing* — a naive
`split("---")` silently eats a user's Markdown horizontal rule, and the failure looks like data
loss with no error anywhere.

- [ ] **Step 4: Register everything, update `current-status.md`, commit**

```bash
git add docs
git commit -m "docs: document weekly reports, the review flow, and file co-ownership"
```

---

## Task 8: Verify the plan's own acceptance

- [x] `npm run check` and `fallow audit` both pass.
- [x] **A4:** opening a review shows the prior report, produces exactly one file for its week, and
      leaves a Weekly Focus ready for the coming week.
- [x] **A8:** achieving a Goal and adjusting next week's focus both work with no review open.
- [x] Typing a reflection, reopening the review, and regenerating leaves the reflection identical
      byte for byte.
- [x] Editing a report file in an external editor, then reopening the review, preserves the edit.
- [x] Writing a `---` horizontal rule inside a report body survives a regeneration.
- [x] Correcting a check-in weeks later changes the old week's summary on reopen.
- [x] The rendered summary contains no `%`, `streak`, or `score`.

**Next:** [0007-ui-surfaces.md](../0007-ui-surfaces.md) and [0008-launcher.md](../0008-launcher.md).
