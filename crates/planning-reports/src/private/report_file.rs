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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
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
