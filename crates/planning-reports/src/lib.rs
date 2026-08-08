//! Weekly Report files: app-owned front matter and one app-owned body region
//! inside an otherwise user-owned Markdown document. This crate knows nothing
//! about Tasks, Habits, or the database.

mod private;

pub use private::document::ReportDocument;
pub use private::error::ReportError;
pub use private::front_matter::ReportFrontMatter;
pub use private::report_file::{SaveBody, WeeklyReportFile, WriteReport};
pub use private::summary_block::SummaryBlock;
